package proggen

import (
	"github.com/google/syzkaller/pkg/log"
	"github.com/google/syzkaller/prog"
	"github.com/shankarapailoor/trace2syz/parser"
)

type structHandler func(syzType *prog.StructType, traceType parser.IrType, ctx *Context) parser.IrType

var specialStructMap = map[string]structHandler{
	"bpf_framed_program": bpfFramedProgramHandler,
}

func preprocessStruct(syzType *prog.StructType, traceType parser.IrType, ctx *Context) parser.IrType {
	if structFunc, ok := specialStructMap[syzType.Name()]; ok {
		return structFunc(syzType, traceType, ctx)
	}
	return traceType
}

func bpfFramedProgramHandler(syzType *prog.StructType, traceType parser.IrType, _ *Context) parser.IrType {
	switch a := traceType.(type) {
	case *parser.GroupType:
		if a.Len > 1 {
			straceStructArgs := make([]parser.IrType, len(syzType.Fields))
			straceStructArgs[1] = a
			straceArg0 := parser.GenDefaultIrType(syzType.Fields[0])
			straceStructArgs[0] = straceArg0
			straceStructArgs = append(straceStructArgs, parser.GenDefaultIrType(syzType.Fields[1]))
			return &parser.GroupType{Elems: straceStructArgs, Len: len(straceStructArgs)}
		}
		log.Fatalf("Failed to parse bpfFramedProgramHandler. Strace array needs at least 2 elements")
	}
	return traceType
}
