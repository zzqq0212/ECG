package proggen

import (
	"fmt"
	"github.com/google/syzkaller/pkg/log"
	"github.com/google/syzkaller/prog"
	"github.com/shankarapailoor/trace2syz/parser"
	"strings"
)

type pair struct {
	A string
	B string
}
type sock struct {
	domain   uint64
	level    uint64
	protocol uint64
}

// CallVariantMap maps system calls to their variants (system calls with $ like socket$packet)
// Keys represent the parts of the system call that need to be used
// to identify the variant. Calls like socket require all three arguments
// Some require two like setosckopt
type CallVariantMap struct {
	Fcntl           map[uint64]string
	Bpf             map[uint64]string
	Socket          map[sock]string
	SocketPair      map[sock]string
	Ioctl           map[pair]string
	GetSetsockopt   map[pair]string
	ConnectionCalls map[string]string //accept, bind, connect
}

func buildVariantMap1Field(variants []*prog.Syscall, callMap map[uint64]string, idx int) {
	for _, variant := range variants {
		switch a := variant.Args[idx].(type) {
		case *prog.ConstType:
			callMap[a.Val] = variant.Name
		case *prog.FlagsType:
			for _, val := range a.Vals {
				if _, ok := callMap[val]; !ok {
					callMap[val] = variant.Name
				}
			}
		}
	}
}

func addSocketOrPair(socketOrPairMap map[sock]string, key sock, val string, target *prog.Target) {
	level := key.level
	socketOrPairMap[key] = val
	key.level = level | target.ConstMap["SOCK_CLOEXEC"]
	socketOrPairMap[key] = val
	key.level = level | target.ConstMap["SOCK_NONBLOCK"]
	socketOrPairMap[key] = val
	key.level |= target.ConstMap["SOCK_CLOEXEC"]
	socketOrPairMap[key] = val
}

//We unfortunately have to maintain separate maps for socketpair and socket. It seems that there are cases where
//socketpair can have multiple variants for the same hash code but socket only has one variant. E.g.
//socketpair$inet_tcp has the following arguments
//      domain const[AF_INET], type const[SOCK_STREAM], proto const[0], fds ptr[out, tcp_pair]
//and socketpair$nbd has the same first three:
//      domain const[AF_INET], type const[SOCK_STREAM], proto const[0], fds ptr[out, nbd_sock_pair]
// If we keep the variant maps the same for both calls then we may choose the variant $nbd for sockets. However,
// such a variant doesn't exist so we should keep these variants separate.
// However, both socket and socketpair are accessed the same way and the maps are virtually the same so this function
// aggregates the logic.
func buildSocketOrPairMap(socketOrPairMap map[sock]string, variants []*prog.Syscall, target *prog.Target) {
	for _, variant := range variants {
		suffix := strings.Split(variant.Name, "$")[1]
		key := sock{}
		switch a := variant.Args[0].(type) {
		case *prog.ConstType:
			key.domain = a.Val
		}
		switch a := variant.Args[2].(type) {
		case *prog.ConstType:
			key.protocol = a.Val
		default:
			//Essentially ignore this key if it is not constant and just choose
			//the entry in the map that matches the first two arguments
			key.protocol = ^uint64(0)
		}
		switch a := variant.Args[1].(type) {
		case *prog.ConstType:
			key.level = a.Val
			addSocketOrPair(socketOrPairMap, key, suffix, target)
		case *prog.FlagsType:
			for _, val := range a.Vals {
				key.level = val
				if _, ok := socketOrPairMap[key]; !ok {
					addSocketOrPair(socketOrPairMap, key, suffix, target)
				}
			}
		}
	}
}

func (c *CallVariantMap) buildIoctlMap(variants []*prog.Syscall) {
	for _, variant := range variants {
		resourceName := variant.Args[0].(*prog.ResourceType).TypeName
		var p pair
		switch a := variant.Args[1].(type) {
		case *prog.ConstType:
			p.A = resourceName
			p.B = fmt.Sprint(a.Val)
			c.Ioctl[p] = strings.Split(variant.Name, "$")[1]
		case *prog.FlagsType:
			for _, val := range a.Vals {
				p.A = resourceName
				p.B = fmt.Sprint(val)
				if _, ok := c.Ioctl[p]; !ok {
					c.Ioctl[p] = strings.Split(variant.Name, "$")[1]
				}
			}
		}

	}
}

func (c *CallVariantMap) buildGetSetsockoptMap(variants []*prog.Syscall) {
	for _, variant := range variants {
		level := variant.Args[1].(*prog.ConstType).Val
		switch a := variant.Args[2].(type) {
		case *prog.FlagsType:
			var p pair
			for _, val := range a.Vals {
				p.A = fmt.Sprint(level)
				p.B = fmt.Sprint(val)
				if _, ok := c.GetSetsockopt[p]; !ok {
					c.GetSetsockopt[p] = strings.Split(variant.Name, "$")[1]
				}
			}
		case *prog.ConstType:
			var p pair
			p.A = fmt.Sprint(variant.Args[1].(*prog.ConstType).Val)
			p.B = fmt.Sprint(a.Val)
			c.GetSetsockopt[p] = strings.Split(variant.Name, "$")[1]
		}
	}
}

func (c *CallVariantMap) buildConnectCallMap(variants []*prog.Syscall) {
	for _, variant := range variants {
		resourceName := variant.Args[0].(*prog.ResourceType).TypeName
		c.ConnectionCalls[resourceName] = strings.Split(variant.Name, "$")[1]
	}
}

// Build constructs the variant mappings from Syzkaller target
func (c *CallVariantMap) Build(target *prog.Target) {
	callVariants := make(map[string][]*prog.Syscall)
	for _, call := range target.Syscalls {
		if strings.Contains(call.Name, "$") {
			if _, ok := callVariants[call.CallName]; !ok {
				callVariants[call.CallName] = []*prog.Syscall{}
			}
			callVariants[call.CallName] = append(callVariants[call.CallName], call)
		}
	}

	for call, variants := range callVariants {
		switch call {
		case "socket":
			buildSocketOrPairMap(c.Socket, variants, target)
		case "socketpair":
			buildSocketOrPairMap(c.SocketPair, variants, target)
		case "ioctl":
			c.buildIoctlMap(variants)
		case "bpf":
			buildVariantMap1Field(variants, c.Bpf, 0)
		case "fcntl":
			buildVariantMap1Field(variants, c.Fcntl, 1)
		case "getsockopt", "setsockopt":
			c.buildGetSetsockoptMap(variants)
		case "accept", "bind", "connect", "accept4", "recvfrom", "sendto", "getsockname":
			c.buildConnectCallMap(variants)
		}
	}
}

// NewCall2VariantMap initializes the variant mapper
func NewCall2VariantMap() (c *CallVariantMap) {
	return &CallVariantMap{
		Fcntl:           make(map[uint64]string),
		Bpf:             make(map[uint64]string),
		Socket:          make(map[sock]string),
		SocketPair:      make(map[sock]string),
		Ioctl:           make(map[pair]string),
		GetSetsockopt:   make(map[pair]string),
		ConnectionCalls: make(map[string]string),
	}
}

type preprocessHook func(ctx *Context)

func preprocess(ctx *Context) {
	call := ctx.CurrentStraceCall.CallName
	if procFunc, ok := preprocessMap[call]; ok {
		procFunc(ctx)
	}
}

var preprocessMap = map[string]preprocessHook{
	"bpf":         bpf,
	"accept":      connectCalls,
	"accept4":     connectCalls,
	"bind":        connectCalls,
	"connect":     connectCalls,
	"fcntl":       fcntl,
	"getsockname": connectCalls,
	"getsockopt":  getSetsockoptCalls,
	"ioctl":       ioctl,
	"open":        open,
	"prctl":       prctl,
	"recvfrom":    connectCalls,
	"mknod":       mknod,
	"modify_ldt":  modifyLdt,
	"openat":      openat,
	"sendto":      connectCalls,
	"setsockopt":  getSetsockoptCalls,
	"shmctl":      shmctl,
	"socket":      socket,
	"socketpair":  socketpair,
	"shmget":      shmget,
}

func bpf(ctx *Context) {
	val := ctx.CurrentStraceCall.Args[0].(parser.Expression).Eval(ctx.Target)
	ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[ctx.Call2Variant.Bpf[val]]
}

func socket(ctx *Context) {
	val1 := ctx.CurrentStraceCall.Args[0].(parser.Expression).Eval(ctx.Target)
	val2 := ctx.CurrentStraceCall.Args[1].(parser.Expression).Eval(ctx.Target)
	val3 := ctx.CurrentStraceCall.Args[2].(parser.Expression).Eval(ctx.Target)
	key := sock{val1, val2, val3}
	if _, ok := ctx.Call2Variant.Socket[key]; !ok {
		key.protocol = ^uint64(0)
	}
	if suffix, ok := ctx.Call2Variant.Socket[key]; ok {
		name := ctx.CurrentStraceCall.CallName + "$" + suffix
		ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[name]
	}
}

func socketpair(ctx *Context) {
	val1 := ctx.CurrentStraceCall.Args[0].(parser.Expression).Eval(ctx.Target)
	val2 := ctx.CurrentStraceCall.Args[1].(parser.Expression).Eval(ctx.Target)
	val3 := ctx.CurrentStraceCall.Args[2].(parser.Expression).Eval(ctx.Target)
	key := sock{val1, val2, val3}
	if _, ok := ctx.Call2Variant.SocketPair[key]; !ok {
		key.protocol = ^uint64(0)
	}
	if suffix, ok := ctx.Call2Variant.SocketPair[key]; ok {
		name := ctx.CurrentStraceCall.CallName + "$" + suffix
		ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[name]
	}
}

func ioctl(ctx *Context) {
	var arg prog.Arg
	var fdType string
	straceFd := ctx.CurrentStraceCall.Args[0]
	syzFd := ctx.CurrentSyzCall.Meta.Args[0]
	if arg = ctx.ReturnCache.get(syzFd, straceFd); arg == nil {
		return
	}
	switch a := arg.Type().(type) {
	case *prog.ResourceType:
		// Start with most descriptive type and see if there is a match
		// Then work backwards to more general resource types
		var suffix string
		var p pair
		for i := len(a.Desc.Kind) - 1; i > -1; i-- {
			fdType = a.Desc.Kind[i]
			val := ctx.CurrentStraceCall.Args[1].(parser.Expression).Eval(ctx.Target)
			p.A = fdType
			p.B = fmt.Sprint(val)
			if suffix = ctx.Call2Variant.Ioctl[p]; suffix == "" {
				continue
			}
			syzName := ctx.CurrentStraceCall.CallName + "$" + suffix
			ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[syzName]
			return
		}
	}
}

func fcntl(ctx *Context) {
	cmd := ctx.CurrentStraceCall.Args[1].(parser.Expression)
	val := cmd.Eval(ctx.Target)
	if name, ok := ctx.Call2Variant.Fcntl[val]; ok {
		ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[name]
	}
}

func connectCalls(ctx *Context) {
	// Connection system calls can take on many subforms such as
	// accept$inet
	// bind$inet6

	// In order to determine the proper form we need to look at the file descriptor to determine
	// the proper socket type. We refer to the $inet as a suffix to the name
	straceFd := ctx.CurrentStraceCall.Args[0]
	syzFd := ctx.CurrentSyzCall.Meta.Args[0]
	var arg prog.Arg
	if arg = ctx.ReturnCache.get(syzFd, straceFd); arg == nil {
		return
	}
	switch a := arg.Type().(type) {
	case *prog.ResourceType:
		// Start with most descriptive type and see if there is a match
		// Then work backwards to more general resource types
		var suffix string
		for i := len(a.Desc.Kind) - 1; i > -1; i-- {
			if suffix = ctx.Call2Variant.ConnectionCalls[a.Desc.Kind[i]]; suffix == "" {
				continue
			}
			syzName := ctx.CurrentStraceCall.CallName + "$" + suffix
			ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[syzName]
			return
		}
	}
}

func getSetsockoptCalls(ctx *Context) {
	sockLevel := ctx.CurrentStraceCall.Args[1].(parser.Expression)
	optName := ctx.CurrentStraceCall.Args[2].(parser.Expression)
	p := pair{
		A: fmt.Sprint(sockLevel.Eval(ctx.Target)),
		B: fmt.Sprint(optName.Eval(ctx.Target)),
	}
	if suffix, ok := ctx.Call2Variant.GetSetsockopt[p]; ok {
		syzName := ctx.CurrentStraceCall.CallName + "$" + suffix
		ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[syzName]
	}
}

func open(ctx *Context) {
	if len(ctx.CurrentStraceCall.Args) >= 3 {
		return
	}
	ctx.CurrentStraceCall.Args = append(ctx.CurrentStraceCall.Args,
		parser.NewIntsType([]int64{0}))
}

func mknod(ctx *Context) {
	if len(ctx.CurrentStraceCall.Args) >= 3 {
		return
	}
	ctx.CurrentStraceCall.Args = append(ctx.CurrentStraceCall.Args,
		parser.NewIntsType([]int64{0}))
}

func openat(ctx *Context) {
	if len(ctx.CurrentSyzCall.Args) >= 4 {
		return
	}
	ctx.CurrentStraceCall.Args = append(ctx.CurrentStraceCall.Args,
		parser.NewIntsType([]int64{0}))
}

func prctl(ctx *Context) {
	prctlCmd := ctx.CurrentStraceCall.Args[0].String()
	variantName := ctx.CurrentStraceCall.CallName + "$" + prctlCmd
	if _, ok := ctx.Target.SyscallMap[variantName]; !ok {
		return
	}
	ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[variantName]
}

func shmctl(ctx *Context) {
	shmctlCmd := ctx.CurrentStraceCall.Args[1].String()
	variantName := ctx.CurrentStraceCall.CallName + "$" + shmctlCmd
	if _, ok := ctx.Target.SyscallMap[variantName]; !ok {
		return
	}
	ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[variantName]
}

func modifyLdt(ctx *Context) {
	suffix := ""
	switch a := ctx.CurrentStraceCall.Args[0].(type) {
	case parser.Expression:
		switch a.Eval(ctx.Target) {
		case 0:
			suffix = "$read"
		case 1:
			suffix = "$write"
		case 2:
			suffix = "$read_default"
		case 17:
			suffix = "$write2"
		}
	default:
		log.Fatalf("Preprocess modifyldt received unexpected strace type: %s\n", a.Name())
	}
	ctx.CurrentStraceCall.CallName = ctx.CurrentStraceCall.CallName + suffix
	ctx.CurrentSyzCall.Meta = ctx.Target.SyscallMap[ctx.CurrentStraceCall.CallName]
}

func shmget(ctx *Context) {
	if ctx.CurrentStraceCall.Ret <= 0 {
		// We have a successful shmget
		return
	}
	switch a := ctx.CurrentStraceCall.Args[1].(type) {
	case parser.Expression:
		size := a.Eval(ctx.Target)
		ctx.Tracker.addShmRequest(uint64(ctx.CurrentStraceCall.Ret), size)
	default:
		log.Fatalf("shmctl could not evaluate size of buffer: %#v\n", a)
	}
}
