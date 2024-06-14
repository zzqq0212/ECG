package proggen

import (
	"fmt"
	"github.com/google/syzkaller/pkg/log"
	"github.com/google/syzkaller/prog"
	_ "github.com/google/syzkaller/sys"
	"github.com/shankarapailoor/trace2syz/parser"
	"strings"
	"testing"
)

var (
	OS   = "linux"
	Arch = "amd64"
)

func parseSingleTrace(t *testing.T, data string) *Context {
	var err error
	var target *prog.Target
	var traceTree *parser.TraceTree
	var ctx *Context

	target, err = prog.GetTarget(OS, Arch)
	if err != nil {
		t.Fatalf("Failed to load target %s\n", err)
	}
	variantMap := NewCall2VariantMap()
	variantMap.Build(target)
	traceTree = parser.ParseLoop(data)
	ctx = GenSyzProg(traceTree.TraceMap[traceTree.RootPid], target, variantMap)

	ctx.FillOutMemory()
	if err = ctx.Prog.Validate(); err != nil {
		t.Fatalf("Failed to parse trace: %s", err.Error())
	}
	return ctx
}

func testMatchingCallSequence(calls []*prog.Call, seq []string) error {
	if len(seq) != len(calls) {
		for _, call := range calls {
			log.Logf(0, "call: %s", call.Meta.Name)
		}
		return fmt.Errorf("call sequence and p.Calls do not have same length %d != %d", len(seq), len(calls))
	}
	for i, call := range calls {
		if call.Meta.Name != seq[i] {
			return fmt.Errorf("Mismatched %s != %s", seq[i], call.Meta.Name)
		}
	}
	return nil
}

func TestParseTraceBasic(t *testing.T) {
	test := `open("file", O_CREAT|O_RDWR) = 3` + "\n" +
		`write(3, "somedata", 8) = 8`
	ctx := parseSingleTrace(t, test)
	p := ctx.Prog
	expectedSeq := []string{"mmap", "open", "write"}
	err := testMatchingCallSequence(p.Calls, expectedSeq)
	if err != nil {
		t.Fatalf("%s", err)
	}
	switch a := p.Calls[2].Args[0].(type) {
	case *prog.ResultArg:
	default:
		t.Fatalf("Expected result arg. Got: %s\n", a.Type().Name())
	}
}

func TestParseTraceInnerResource(t *testing.T) {
	test := `pipe([5,6]) = 0` + "\n" +
		`write(6, "\xff\xff\xfe\xff", 4) = 4`

	p := parseSingleTrace(t, test).Prog
	expectedSeq := []string{"mmap", "pipe", "write"}
	err := testMatchingCallSequence(p.Calls, expectedSeq)
	if err != nil {
		t.Fatalf("%s", err)
	}
	switch a := p.Calls[2].Args[0].(type) {
	case *prog.ResultArg:
	default:
		t.Fatalf("Expected result arg. Got: %s\n", a.Type().Name())
	}
}

func TestDistinguishResourceTypes(t *testing.T) {
	test := `inotify_init() = 2` + "\n" +
		`open("tmp", O_RDONLY|O_CLOEXEC) = 3` + "\n" +
		`inotify_add_watch(3, "\x2e", 0xfff) = 3` + "\n" +
		`write(3, "temp", 5) = 5` + "\n" +
		`inotify_rm_watch(2, 3) = 0`
	expected := []string{"mmap", "inotify_init", "open", "inotify_add_watch", "write", "inotify_rm_watch"}
	p := parseSingleTrace(t, test).Prog
	err := testMatchingCallSequence(p.Calls, expected)
	if err != nil {
		t.Fatalf("%s\n", err)
	}
	write := p.Calls[len(p.Calls)-2]
	inotifyRmWatch := p.Calls[len(p.Calls)-1]
	switch a := write.Args[0].Type().(type) {
	case *prog.ResourceType:
		if a.TypeName != "fd" {
			t.Fatalf("Expected first argument of write to have type fd. Got: %s", a.TypeName)
		}
	default:
		t.Fatalf("First argument of write is not resource type. Is: %s", a.Name())
	}
	switch a := inotifyRmWatch.Args[1].Type().(type) {
	case *prog.ResourceType:
		if a.TypeName != "inotifydesc" {
			t.Fatalf("Expected second argument of inotify_rm_watch to have type inoitfydesc. Got: %s", a.TypeName)
		}
	default:
		t.Fatalf("First argument of write is not resource type. Is: %s", a.Name())
	}
}

func TestSocketLevel(t *testing.T) {
	test := `socket(AF_UNIX, SOCK_STREAM, 0) = 3` + "\n" +
		`socket(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK, 0) = 3` + "\n" +
		`socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0) = 3` + "\n" +
		`socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0) = 3` + "\n"
	expected := []string{"socket$unix", "socket$unix", "socket$unix", "socket$unix"}
	p := parseSingleTrace(t, test).Prog
	err := testMatchingCallSequence(p.Calls, expected)
	if err != nil {
		t.Fatalf("%s\n", err)
	}
}

func TestIdentifySockaddrStorage(t *testing.T) {
	type callStorageParams struct {
		CallIdx   int
		ArgIdx    int
		FieldName string
	}
	test1 := `open("temp", O_WRONLY) = 3` + "\n" +
		`connect(3, {sa_family=AF_INET, sin_port=htons(37957), sin_addr=inet_addr("0.0.0.0")}, 16) = -1`
	test2 := `open("temp", O_WRONLY) = 3` + "\n" +
		`connect(6, {sa_family=AF_INET6, sin6_port=htons(8888), inet_pton(AF_INET6, "::1", &sin6_addr),` +
		`sin6_flowinfo=htonl(4286513152), sin6_scope_id=0}, 128) = 0`
	test3 := `open("temp", O_WRONLY) = 3` + "\n" +
		`connect(3, {sa_family=AF_UNIX, sun_path="temp"}, 110) = -1`
	test4 := `open("temp", O_WRONLY) = 3` + "\n" +
		`bind(5, {sa_family=AF_NETLINK, nl_pid=0, nl_groups=00000000}, 12)  = -1`
	tests := []string{test1, test2, test3, test4}
	expected := [][]string{
		{"mmap", "open", "connect"},
		{"mmap", "open", "connect"},
		{"mmap", "open", "connect"},
		{"mmap", "open", "bind"},
	}
	checkParams := []callStorageParams{
		{2, 1, "sockaddr_in"},
		{2, 1, "sockaddr_in6"},
		{2, 1, "sockaddr_un"},
		{2, 1, "sockaddr_nl"},
	}

	validator := func(arg prog.Arg, field string) error {
		var (
			storageArg *prog.UnionArg
			storagePtr *prog.PointerArg
			ok         bool
		)
		storagePtr = arg.(*prog.PointerArg)
		if storageArg, ok = storagePtr.Res.(*prog.UnionArg); !ok {
			t.Fatalf("Second argument not union. Type: %s", storagePtr.Res.Type().Name())
		}
		fieldName := storageArg.Option.Type().Name()
		if fieldName != field {
			return fmt.Errorf("Incorrect storage type. Expected %s != %s", field, fieldName)
		}
		return nil
	}

	for i, test := range tests {
		p := parseSingleTrace(t, test).Prog
		err := testMatchingCallSequence(p.Calls, expected[i])
		if err != nil {
			t.Fatalf("Failed to parse calls: %s", err)
		}
		param := checkParams[i]
		err = validator(p.Calls[param.CallIdx].Args[param.ArgIdx], param.FieldName)
		if err != nil {
			t.Fatalf("Failed to infer sockaddr union for test: %d with err: %s", i, err)
		}
	}
}

func TestIdentifyIfru(t *testing.T) {
	test := `socket(AF_PACKET, SOCK_RAW, 768)  = 3` + "\n" +
		`ioctl(3, SIOCGIFHWADDR, {ifr_name="\x6c\x6f", ifr_hwaddr=00:00:00:00:00:00}) = 0`
	tests := []string{test}
	expected := [][]string{
		{"mmap", "socket$packet", "ioctl$sock_ifreq"},
	}
	for i, test := range tests {
		p := parseSingleTrace(t, test).Prog
		err := testMatchingCallSequence(p.Calls, expected[i])
		if err != nil {
			t.Fatalf("error %s on test %d.", err, i)
		}
	}
}

func TestParseVariants(t *testing.T) {
	test1 := `socket(AF_UNIX, SOCK_STREAM, 0) = 3` + "\n" +
		`connect(3, {sa_family=AF_UNIX, sun_path="temp"}, 110) = -1 ENOENT (Bad file descriptor)`
	test2 := `socket(AF_UNIX, SOCK_STREAM, 0) = 3`
	test3 := `socket(AF_INET, SOCK_STREAM, IPPROTO_IP) = 5` + "\n" +
		`ioctl(5, FIONBIO, [1]) = 0`
	test4 := `socket(AF_INET, SOCK_STREAM, IPPROTO_IP) = 3` + "\n" +
		`connect(3, {sa_family=AF_INET, sin_port=htons(37957), sin_addr=inet_addr("0.0.0.0")}, 16) = 0`
	test5 := `socket(AF_INET, SOCK_STREAM, IPPROTO_IP) = 3` + "\n" +
		`setsockopt(3, SOL_SOCKET, SO_REUSEADDR, [1], 4) = 0`
	test6 := `9795  socket(AF_PACKET, SOCK_RAW, 768)  = 3` + "\n" +
		`9795  ioctl(3, SIOCGIFINDEX, {ifr_name="\x6c\x6f", }) = 0`
	test7 := `open("temp", O_WRONLY) = 3` + "\n" +
		`connect(3, {sa_family=AF_INET, sin_port=htons(37957), sin_addr=inet_addr("0.0.0.0")}, 16) = -1`
	tests := []string{test1, test2, test3, test4, test5, test6, test7}
	expected := [][]string{
		{"mmap", "socket$unix", "connect$unix"},
		{"socket$unix"},
		{"mmap", "socket$inet_tcp", "ioctl$int_in"},
		{"mmap", "socket$inet_tcp", "connect$inet"},
		{"mmap", "socket$inet_tcp", "setsockopt$sock_int"},
		{"mmap", "socket$packet", "ioctl$sock_SIOCGIFINDEX"},
		{"mmap", "open", "connect"},
	}
	for i, test := range tests {
		p := parseSingleTrace(t, test).Prog
		err := testMatchingCallSequence(p.Calls, expected[i])
		if err != nil {
			t.Fatalf("error %s on test %d.", err, i)
		}
	}
}

func TestParseIPv4(t *testing.T) {
	test1 := `socket(AF_INET, SOCK_STREAM, IPPROTO_IP) = 3` + "\n" +
		`connect(3, {sa_family=AF_INET, sin_port=htons(37957), sin_addr=inet_addr("0.0.0.0")}, 16) = 0`
	test2 := `socket(AF_INET, SOCK_STREAM, IPPROTO_IP) = 3` + "\n" +
		`connect(3, {sa_family=AF_INET, sin_port=htons(37957), sin_addr=inet_addr("127.0.0.1")}, 16) = 0`
	tests := []string{test1, test2}
	expected := [][]string{
		{"mmap", "socket$inet_tcp", "connect$inet"},
		{"mmap", "socket$inet_tcp", "connect$inet"},
	}
	expectedIps := []uint64{0x0, 0x7f000001}
	testIpv4 := func(expectedIp uint64, a prog.Arg, t *testing.T) {
		sockaddr, ok := a.(*prog.PointerArg).Res.(*prog.GroupArg)
		if !ok {
			t.Fatalf("%s", a.Type().Name())
		}
		ipv4Addr, ok := sockaddr.Inner[2].(*prog.UnionArg)
		if !ok {
			t.Fatalf("Expected 3rd argument to be unionArg. Got %s", sockaddr.Inner[2].Type().Name())
		}
		optName := ipv4Addr.Option.Type().FieldName()
		if !strings.Contains(optName, "rand") {
			t.Fatalf("Expected ip option to be random opt. Got: %s", optName)
		}
		ip, ok := ipv4Addr.Option.(*prog.ConstArg)
		if !ok {
			t.Fatalf("ipv4Addr option is not IntType")
		}
		if ip.Val != expectedIp {
			t.Fatalf("Parsed != Expected, %d != %d", ip.Val, expectedIp)
		}
	}
	for i, test := range tests {
		p := parseSingleTrace(t, test).Prog
		err := testMatchingCallSequence(p.Calls, expected[i])
		if err != nil {
			t.Fatalf("Failed test: %d with error: %s", i, err)
		}
		testIpv4(expectedIps[i], p.Calls[2].Args[1], t)
	}
}
