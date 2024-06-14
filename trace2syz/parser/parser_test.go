package parser

import (
	"github.com/google/syzkaller/prog"
	_ "github.com/google/syzkaller/sys"
	"testing"
)

var (
	OS   = "linux"
	Arch = "amd64"
)

func TestParseLoopBasic(t *testing.T) {
	tests := []string{
		`open() = 3` + "\n" +
			`fstat() = 0`,
		`open() = 0x73ffddabc` + "\n" +
			`fstat() = 0`,
		`open() = -1 ENOSPEC (something)` + "\n" +
			`fstat() = 0`,
		`open( ,  <unfinished ...>` + "\n" +
			`<... open resumed>) = 3` + "\n" +
			`fstat() = 0`,
		`open( ,  <unfinished ...>` + "\n" +
			`<... open resumed> , 2) = 3` + "\n" +
			`fstat() = 0`,
		`open( <unfinished ...>` + "\n" +
			`<... open resumed>) = 3` + "\n" +
			`fstat() = 0`,
		`open( <unfinished ...>` + "\n" +
			`<... open resumed>) = 0x44277ffff` + "\n" +
			`fstat() = 0`,
		`open( <unfinished ...>` + "\n" +
			`<... open resumed>) = ?` + "\n" +
			`fstat() = 0`,
		`open( <unfinished ...>` + "\n" +
			`<... open resumed>) = -1 FLAG (sdfjfjfjf)` + "\n" +
			`fstat() = 0`,
		`open(1,  <unfinished ...>` + "\n" +
			`<... open resumed> , FLAG|FLAG) = -1 FLAG (sdfjfjfjf)` + "\n" +
			`fstat() = 0`,
		`open([USR1 IO], NULL, {tv_sec=5, tv_nsec=0}, 8 <unfinished ...>` + "\n" +
			`<... rt_sigtimedwait resumed> )   = 10 (SIGUSR1)` + "\n" +
			`fstat() = 0`,
		`open(0, SNDCTL_TMR_START or TCSETS,` +
			`{c_cc[VMIN]=1, c_cc[VTIME]=0} <unfinished ...>` + "\n" +
			`<... open resumed> , FLAG|FLAG) = -1 FLAG (sdfjfjfjf)` + "\n" +
			`fstat() = 0`,
	}

	for _, test := range tests {
		tree := ParseLoop(test)
		if tree.RootPid != -1 {
			t.Fatalf("Incorrect Root Pid: %d\n", tree.RootPid)
		}

		calls := tree.TraceMap[tree.RootPid].Calls
		if len(calls) != 2 {
			t.Fatalf("Expect 2 calls. Got %d instead", len(calls))
		}
		if calls[0].CallName != "open" || calls[1].CallName != "fstat" {
			t.Fatalf("call list should be open->fstat. Got %s->%s\n", calls[0].CallName, calls[1].CallName)
		}
	}

}

func TestParseLoopPid(t *testing.T) {
	/*
		Parses two basic calls. Make sure the trace tree just has one entry with two calls
	*/

	data := `1  open() = 3` + "\n" +
		`1  fstat() = 0`

	tree := ParseLoop(data)
	if tree.RootPid != 1 {
		t.Fatalf("Incorrect Root Pid: %d\n", tree.RootPid)
	}

	calls := tree.TraceMap[tree.RootPid].Calls
	if len(calls) != 2 {
		t.Fatalf("Expect 2 calls. Got %d instead", len(calls))
	}
	if calls[0].CallName != "open" || calls[1].CallName != "fstat" {
		t.Fatalf("call list should be open->fstat. Got %s->%s\n", calls[0].CallName, calls[1].CallName)
	}
}

func TestParseLoop1Child(t *testing.T) {
	data1Child := `1 open() = 3` + "\n" +
		`1 clone() = 2` + "\n" +
		`2 read() = 16`

	tree := ParseLoop(data1Child)
	if len(tree.Ptree) != 2 {
		t.Fatalf("Incorrect Root Pid. Expected: 2, Got %d\n", tree.RootPid)
	}
	if tree.Ptree[tree.RootPid][0] != 2 {
		t.Fatalf("Expected child to have pid: 2. Got %d\n", tree.Ptree[tree.RootPid][0])
	} else {
		if len(tree.TraceMap[2].Calls) != 1 {
			t.Fatalf("Child trace should have only 1 call. Got %d\n", len(tree.TraceMap[2].Calls))
		}
	}
}

func TestParseLoop2Childs(t *testing.T) {
	data2Childs := `1 open() = 3` + "\n" +
		`1 clone() = 2` + "\n" +
		`2 read() = 16` + "\n" +
		`1 clone() = 3` + "\n" +
		`3 open() = 3`
	tree := ParseLoop(data2Childs)
	if len(tree.Ptree) != 3 {
		t.Fatalf("Incorrect Root Pid. Expected: 3, Got %d\n", tree.RootPid)
	}
	if len(tree.Ptree[tree.RootPid]) != 2 {
		t.Fatalf("Expected Pid 1 to have 2 children: Got %d\n", len(tree.Ptree[tree.RootPid]))
	}
}

func TestParseLoop1Grandchild(t *testing.T) {
	data1Grandchild := `1 open() = 3` + "\n" +
		`1 clone() = 2` + "\n" +
		`2 clone() = 3` + "\n" +
		`3 open() = 4`
	tree := ParseLoop(data1Grandchild)
	if len(tree.Ptree[tree.RootPid]) != 1 {
		t.Fatalf("Expect RootPid to have 1 child. Got %d\n", tree.RootPid)
	}
	if len(tree.Ptree[2]) != 1 {
		t.Fatalf("Incorrect Root Pid. Expected: 3, Got %d\n", tree.RootPid)

	}
}

func TestParseIrTypes(t *testing.T) {
	type irTest struct {
		test   string
		irType string
	}
	test1 := irTest{`open(MAKEDEV(1)) = 0`, exprTypeName}
	test2 := irTest{`open({1, 2, 3}) = 0`, groupTypeName}
	test3 := irTest{`open([1, 2, 3]) = 0`, groupTypeName}
	test4 := irTest{`open([1 2]) = 0`, groupTypeName}
	test5 := irTest{`open(TCSETS or TCGETS) = 0`, exprTypeName}
	tests := []irTest{test1, test2, test3, test4, test5}
	for i, test := range tests {
		tree := ParseLoop(test.test)
		call := tree.TraceMap[tree.RootPid].Calls[0]
		if call.Args[0].Name() != test.irType {
			t.Fatalf("Failed test %d. Expected %s != %s", i, test.irType, call.Args[0].Name())
		}
	}
}

func TestEvalFlags(t *testing.T) {
	target, err := prog.GetTarget(OS, Arch)
	if err != nil {
		t.Fatalf("Failed to load target %s\n", err)
	}
	type desc struct {
		test         string
		expectedEval uint64
	}
	tests := []desc{
		{test: `open(AT_FDCWD) = 0`, expectedEval: 18446744073709551516},
		{test: `open([BUS ALRM IO]) = 0`, expectedEval: 10 | 14 | 23},
		{test: `open([BUS]) = 0`, expectedEval: 10},
	}
	for i, test := range tests {
		tree := ParseLoop(test.test)
		call := tree.TraceMap[tree.RootPid].Calls[0]
		var expr Expression
		switch a := call.Args[0].(type) {
		case *GroupType:
			expr = a.Elems[0].(Expression)
		case Expression:
			expr = a
		}
		flagEval := expr.Eval(target)
		if test.expectedEval != flagEval {
			t.Fatalf("Incorrect Flag Evaluation for Test %d. Expected %v != %v", i, test.expectedEval, flagEval)
		}
	}
}
