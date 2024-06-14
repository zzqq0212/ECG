package proggen

import (
	"encoding/binary"
	"fmt"
	"github.com/google/syzkaller/pkg/log"
	"github.com/google/syzkaller/prog"
	"github.com/shankarapailoor/trace2syz/parser"
	"github.com/shankarapailoor/trace2syz/utils"
	"math/rand"
	"strings"
)

type returnCache map[resourceDescription]prog.Arg

func newRCache() returnCache {
	return make(map[resourceDescription]prog.Arg)
}

func (r *returnCache) buildKey(syzType prog.Type) string {
	switch a := syzType.(type) {
	case *prog.ResourceType:
		return "ResourceType-" + a.Desc.Kind[0]
	default:
		log.Fatalf("Caching non resource type")
	}
	return ""
}

func (r *returnCache) cache(syzType prog.Type, traceType parser.IrType, arg prog.Arg) {
	log.Logf(2, "Caching resource type: %s, val: %s", r.buildKey(syzType), traceType.String())
	resDesc := resourceDescription{
		Type: r.buildKey(syzType),
		Val:  traceType.String(),
	}
	(*r)[resDesc] = arg
}

func (r *returnCache) get(syzType prog.Type, traceType parser.IrType) prog.Arg {
	log.Logf(2, "Fetching resource type: %s, val: %s", r.buildKey(syzType), traceType.String())
	resDesc := resourceDescription{
		Type: r.buildKey(syzType),
		Val:  traceType.String(),
	}
	if arg, ok := (*r)[resDesc]; ok {
		if arg != nil {
			log.Logf(2, "Cache hit for resource type: %s, val: %s", r.buildKey(syzType), traceType.String())
			return arg
		}
	}
	return nil
}

type resourceDescription struct {
	Type string
	Val  string
}

// Context stores metadata related to a syzkaller program
// Currently we are embedding the State object within the Context.
// We should probably merge the two objects
type Context struct {
	ReturnCache       returnCache
	Prog              *prog.Prog
	CurrentStraceCall *parser.Syscall
	CurrentSyzCall    *prog.Call
	CurrentStraceArg  parser.IrType
	Target            *prog.Target
	Tracker           *memoryTracker
	CallToCover       map[*prog.Call][]uint64
	Call2Variant      *CallVariantMap
	DependsOn         map[*prog.Call]map[*prog.Call]int
}

func newContext(target *prog.Target, variantMap *CallVariantMap) (ctx *Context) {
	ctx = &Context{}
	ctx.ReturnCache = newRCache()
	ctx.CurrentStraceCall = nil
	ctx.Tracker = newTracker()
	ctx.CurrentStraceArg = nil
	ctx.Target = target
	ctx.CallToCover = make(map[*prog.Call][]uint64)
	ctx.Call2Variant = variantMap
	ctx.DependsOn = make(map[*prog.Call]map[*prog.Call]int)
	return
}

// FillOutMemory determines how much memory to allocate for arguments in a program
// And generates an mmap c to do the allocation.This mmap is prepended to prog.Calls
func (ctx *Context) FillOutMemory() error {
	err := ctx.Tracker.fillOutMemory(ctx.Prog)
	if err != nil {
		return err
	}
	totalMemory := ctx.Tracker.getTotalMemoryAllocations(ctx.Prog)
	log.Logf(2, "Total memory for program is: %d", totalMemory)
	if totalMemory == 0 {
		log.Logf(1, "Program requires no mmaps. Total memory: %d", totalMemory)
		return nil
	}
	mmapCall := ctx.Target.MakeMmap(0, totalMemory)
	calls := make([]*prog.Call, 0)
	calls = append(append(calls, mmapCall), ctx.Prog.Calls...)
	ctx.Prog.Calls = calls
	return nil
}

// GenSyzProg converts a trace to a syzkaller program
func GenSyzProg(trace *parser.Trace, target *prog.Target, variantMap *CallVariantMap) *Context {
	syzProg := new(prog.Prog)
	syzProg.Target = target
	ctx := newContext(target, variantMap)
	ctx.Prog = syzProg
	var call *prog.Call
	for _, sCall := range trace.Calls {
		if sCall.Paused {
			// Probably a case where the call was killed by a signal like the following
			// 2179  wait4(2180,  <unfinished ...>
			// 2179  <... wait4 resumed> 0x7fff28981bf8, 0, NULL) = ? ERESTARTSYS
			// 2179  --- SIGUSR1 {si_signo=SIGUSR1, si_code=SI_USER, si_pid=2180, si_uid=0} ---
			continue
		}
		ctx.CurrentStraceCall = sCall

		if shouldSkip(ctx) {
			log.Logf(3, "Skipping call: %s", ctx.CurrentStraceCall.CallName)
			continue
		}
		if call = genCall(ctx); call == nil {
			continue
		}

		ctx.CallToCover[call] = sCall.Cover
		ctx.Target.AssignSizesCall(call)
		syzProg.Calls = append(syzProg.Calls, call)
	}
	return ctx
}

func genCall(ctx *Context) *prog.Call {
	log.Logf(2, "parsing call: %s", ctx.CurrentStraceCall.CallName)
	straceCall := ctx.CurrentStraceCall
	syzCallDef := ctx.Target.SyscallMap[straceCall.CallName]
	retCall := new(prog.Call)
	retCall.Meta = syzCallDef
	ctx.CurrentSyzCall = retCall

	preprocess(ctx)
	if ctx.CurrentSyzCall.Meta == nil {
		// A call like fcntl may have variants like fcntl$get_flag
		// but no generic fcntl system call in Syzkaller
		return nil
	}
	retCall.Ret = prog.MakeReturnArg(ctx.CurrentSyzCall.Meta.Ret)

	if call := parseMemoryCall(ctx); call != nil {
		ctx.Target.SanitizeCall(call)
		return call
	}
	for i := range retCall.Meta.Args {
		var strArg parser.IrType
		if i < len(straceCall.Args) {
			strArg = straceCall.Args[i]
		}
		res := genArgs(retCall.Meta.Args[i], strArg, ctx)
		retCall.Args = append(retCall.Args, res)
	}
	genResult(retCall.Meta.Ret, straceCall.Ret, ctx)
	ctx.Target.SanitizeCall(retCall)
	return retCall
}

func genResult(syzType prog.Type, straceRet int64, ctx *Context) {
	if straceRet > 0 {
		straceExpr := parser.NewIntsType([]int64{straceRet})
		switch syzType.(type) {
		case *prog.ResourceType:
			log.Logf(2, "Call: %s returned a resource type with val: %s",
				ctx.CurrentStraceCall.CallName, straceExpr.String())
			ctx.ReturnCache.cache(syzType, straceExpr, ctx.CurrentSyzCall.Ret)
		}
	}
}

func genArgs(syzType prog.Type, traceArg parser.IrType, ctx *Context) prog.Arg {
	if traceArg == nil {
		log.Logf(3, "Parsing syzType: %s, traceArg is nil. Generating default arg...", syzType.Name())
		return genDefaultArg(syzType, ctx)
	}
	ctx.CurrentStraceArg = traceArg
	log.Logf(3, "Parsing Arg of syz type: %s, ir type: %s", syzType.Name(), traceArg.Name())

	switch a := syzType.(type) {
	case *prog.IntType, *prog.ConstType, *prog.FlagsType, *prog.CsumType:
		return genConst(a, traceArg, ctx)
	case *prog.LenType:
		return genDefaultArg(syzType, ctx)
	case *prog.ProcType:
		return parseProc(a, traceArg, ctx)
	case *prog.ResourceType:
		return genResource(a, traceArg, ctx)
	case *prog.PtrType:
		return genPtr(a, traceArg, ctx)
	case *prog.BufferType:
		return genBuffer(a, traceArg, ctx)
	case *prog.StructType:
		return genStruct(a, traceArg, ctx)
	case *prog.ArrayType:
		return genArray(a, traceArg, ctx)
	case *prog.UnionType:
		return genUnionArg(a, traceArg, ctx)
	case *prog.VmaType:
		return genVma(a, traceArg, ctx)
	default:
		log.Fatalf("Unsupported  Type: %v", syzType)
	}
	return nil
}

func genVma(syzType *prog.VmaType, traceType parser.IrType, ctx *Context) prog.Arg {
	var npages uint64 = 1
	// TODO: strace doesn't give complete info, need to guess random page range
	if syzType.RangeBegin != 0 || syzType.RangeEnd != 0 {
		npages = syzType.RangeEnd
	}
	arg := prog.MakeVmaPointerArg(syzType, 0, npages)
	ctx.Tracker.addAllocation(ctx.CurrentSyzCall, ctx.Target.PageSize, arg)
	return arg
}

func genArray(syzType *prog.ArrayType, traceType parser.IrType, ctx *Context) prog.Arg {
	var args []prog.Arg
	switch a := traceType.(type) {
	case *parser.GroupType:
		if syzType.Dir() == prog.DirOut {
			return genDefaultArg(syzType, ctx)
		}
		for i := 0; i < a.Len; i++ {
			args = append(args, genArgs(syzType.Type, a.Elems[i], ctx))
		}
	case *parser.Field:
		return genArray(syzType, a.Val, ctx)
	case *parser.PointerType, parser.Expression, *parser.BufferType:
		return genDefaultArg(syzType, ctx)
	default:
		log.Fatalf("Error parsing Array: %s with Wrong Type: %s", syzType.FldName, traceType.Name())
	}
	return prog.MakeGroupArg(syzType, args)
}

func genStruct(syzType *prog.StructType, traceType parser.IrType, ctx *Context) prog.Arg {
	if syzType.Dir() == prog.DirOut {
		return genDefaultArg(syzType, ctx)
	}
	traceType = preprocessStruct(syzType, traceType, ctx)
	args := make([]prog.Arg, 0)
	switch a := traceType.(type) {
	case *parser.GroupType:
		reorderStructFields(syzType, a, ctx)
		args = append(args, evalFields(syzType.Fields, a.Elems, ctx)...)
	case *parser.Field:
		return genArgs(syzType, a.Val, ctx)
	case *parser.Call:
		args = append(args, parseInnerCall(syzType, a, ctx))
	case parser.Expression:
		return genDefaultArg(syzType, ctx)
	case *parser.BufferType:
		return genDefaultArg(syzType, ctx)
	default:
		log.Fatalf("Unsupported Strace Type: %#v to Struct Type", a)
	}
	return prog.MakeGroupArg(syzType, args)
}

func evalFields(syzFields []prog.Type, straceFields []parser.IrType, ctx *Context) []prog.Arg {
	var args []prog.Arg
	j := 0
	for i := range syzFields {
		if prog.IsPad(syzFields[i]) {
			args = append(args, prog.DefaultArg(syzFields[i]))
		} else {
			if j >= len(straceFields) {
				args = append(args, genDefaultArg(syzFields[i], ctx))
			} else {
				args = append(args, genArgs(syzFields[i], straceFields[j], ctx))
			}
			j++
		}
	}
	return args
}

func genUnionArg(syzType *prog.UnionType, straceType parser.IrType, ctx *Context) prog.Arg {
	if straceType == nil {
		log.Logf(1, "Generating union arg. StraceType is nil")
	} else {
		log.Logf(4, "Generating union arg: %s %s", syzType.TypeName, straceType.Name())
	}
	switch strType := straceType.(type) {
	case *parser.Field:
		switch strValType := strType.Val.(type) {
		case *parser.Call:
			return parseInnerCall(syzType, strValType, ctx)
		default:
			return genUnionArg(syzType, strType.Val, ctx)
		}
	case *parser.Call:
		return parseInnerCall(syzType, strType, ctx)
	default:
		idx := identifyUnionType(syzType, ctx, syzType.TypeName)
		innerType := syzType.Fields[idx]
		return prog.MakeUnionArg(syzType, genArgs(innerType, straceType, ctx))
	}
}

func identifyUnionType(syzType *prog.UnionType, ctx *Context, typeName string) int {
	log.Logf(4, "Identifying union arg: %s", syzType.TypeName)
	switch typeName {
	case "sockaddr_storage":
		return identifySockaddrStorage(syzType, ctx)
	case "sockaddr_nl":
		return identifySockaddrNetlinkUnion(syzType, ctx)
	case "ifr_ifru":
		return identifyIfrIfruUnion(ctx)
	case "ifconf":
		return identifyIfconfUnion(ctx)
	case "bpf_instructions":
		return 0
	case "bpf_insn":
		return 1
	}
	return 0
}

func identifySockaddrStorage(syzType *prog.UnionType, ctx *Context) int {
	field2Opt := make(map[string]int)
	for i, field := range syzType.Fields {
		field2Opt[field.FieldName()] = i
	}
	// We currently look at the first argument of the system call
	// To determine which option of the union we select.
	call := ctx.CurrentStraceCall
	var straceArg parser.IrType
	switch call.CallName {
	// May need to handle special cases.
	case "recvfrom":
		straceArg = call.Args[4]
	default:
		if len(call.Args) >= 2 {
			straceArg = call.Args[1]
		} else {
			log.Fatalf("Unable identify union for sockaddr_storage for call: %s",
				call.CallName)
		}
	}
	switch strType := straceArg.(type) {
	case *parser.GroupType:
		for i := range strType.Elems {
			fieldStr := strType.Elems[i].String()
			if strings.Contains(fieldStr, "AF_INET6") {
				return field2Opt["in6"]
			} else if strings.Contains(fieldStr, "AF_INET") {
				return field2Opt["in"]
			} else if strings.Contains(fieldStr, "AF_UNIX") {
				return field2Opt["un"]
			} else if strings.Contains(fieldStr, "AF_NETLINK") {
				return field2Opt["nl"]
			} else {
				log.Fatalf("Unable to identify option for sockaddr storage union."+
					" Field is: %s", fieldStr)
			}
		}
	default:
		log.Fatalf("Failed to parse Sockaddr Stroage Union Type. Strace Type: %#v", strType)
	}
	return -1
}

func identifySockaddrNetlinkUnion(syzType *prog.UnionType, ctx *Context) int {
	field2Opt := make(map[string]int)
	for i, field := range syzType.Fields {
		field2Opt[field.FieldName()] = i
	}
	switch a := ctx.CurrentStraceArg.(type) {
	case *parser.GroupType:
		if len(a.Elems) > 2 {
			switch b := a.Elems[1].(type) {
			case parser.Expression:
				pid := b.Eval(ctx.Target)
				if pid > 0 {
					// User
					return field2Opt["proc"]
				} else if pid == 0 {
					// Kernel
					return field2Opt["kern"]
				} else {
					// Unspec
					return field2Opt["unspec"]
				}
			case *parser.Field:
				curArg := ctx.CurrentStraceArg
				ctx.CurrentStraceArg = b.Val
				idx := identifySockaddrNetlinkUnion(syzType, ctx)
				ctx.CurrentStraceArg = curArg
				return idx
			default:
				log.Fatalf("Parsing netlink addr struct and expect expression for first arg: %s", a.Name())
			}
		}
	}
	return 2
}

func identifyIfrIfruUnion(ctx *Context) int {
	switch ctx.CurrentStraceArg.(type) {
	case parser.Expression:
		return 2
	case *parser.Field:
		return 2
	default:
		return 0
	}
}

func identifyIfconfUnion(ctx *Context) int {
	switch ctx.CurrentStraceArg.(type) {
	case *parser.GroupType:
		return 1
	default:
		return 0
	}
}

func genBuffer(syzType *prog.BufferType, traceType parser.IrType, ctx *Context) prog.Arg {
	if syzType.Dir() == prog.DirOut {
		if !syzType.Varlen() {
			return prog.MakeOutDataArg(syzType, syzType.Size())
		}
		switch a := traceType.(type) {
		case *parser.BufferType:
			return prog.MakeOutDataArg(syzType, uint64(len(a.Val)))
		case *parser.Field:
			return genBuffer(syzType, a.Val, ctx)
		default:
			switch syzType.Kind {
			case prog.BufferBlobRand:
				size := rand.Intn(256)
				return prog.MakeOutDataArg(syzType, uint64(size))

			case prog.BufferBlobRange:
				max := rand.Intn(int(syzType.RangeEnd) - int(syzType.RangeBegin) + 1)
				size := max + int(syzType.RangeBegin)
				return prog.MakeOutDataArg(syzType, uint64(size))
			default:
				panic(fmt.Sprintf("unexpected buffer type kind: %v. call %v arg %v", syzType.Kind, ctx.CurrentSyzCall, traceType))
			}
		}
	}
	var bufVal []byte
	switch a := traceType.(type) {
	case *parser.BufferType:
		bufVal = []byte(a.Val)
	case parser.Expression:
		val := a.Eval(ctx.Target)
		bArr := make([]byte, 8)
		binary.LittleEndian.PutUint64(bArr, val)
		bufVal = bArr
	case *parser.PointerType:
		val := a.Address
		bArr := make([]byte, 8)
		binary.LittleEndian.PutUint64(bArr, val)
		bufVal = bArr
	case *parser.GroupType:
		return genDefaultArg(syzType, ctx)
	case *parser.Field:
		return genArgs(syzType, a.Val, ctx)
	default:
		log.Fatalf("Cannot parse type %#v for Buffer Type\n", traceType)
	}
	if !syzType.Varlen() {
		size := syzType.Size()
		for uint64(len(bufVal)) < size {
			bufVal = append(bufVal, 0)
		}
		bufVal = bufVal[:size]
	}
	return prog.MakeDataArg(syzType, bufVal)
}

func genPtr(syzType *prog.PtrType, traceType parser.IrType, ctx *Context) prog.Arg {
	switch a := traceType.(type) {
	case *parser.PointerType:
		if a.IsNull() {
			return prog.DefaultArg(syzType)
		}
		if a.Res == nil {
			res := genDefaultArg(syzType.Type, ctx)
			return addr(ctx, syzType, res.Size(), res)
		}
		res := genArgs(syzType.Type, a.Res, ctx)
		return addr(ctx, syzType, res.Size(), res)

	case parser.Expression:
		// Likely have a type of the form bind(3, 0xfffffffff, [3]);
		res := genDefaultArg(syzType.Type, ctx)
		return addr(ctx, syzType, res.Size(), res)
	case *parser.Field:
		return genPtr(syzType, a.Val, ctx)
	default:
		res := genArgs(syzType.Type, a, ctx)
		return addr(ctx, syzType, res.Size(), res)
	}
}

func genConst(syzType prog.Type, traceType parser.IrType, ctx *Context) prog.Arg {
	if syzType.Dir() == prog.DirOut {
		return prog.DefaultArg(syzType)
	}
	switch a := traceType.(type) {
	case parser.Expression:
		switch b := a.(type) {
		case parser.Ints:
			if len(b) >= 2 {
				// May get here through select. E.g. select(2, [6, 7], ..) since Expression can
				// be Ints. However, creating fd set is hard and we let default arg through
				return genDefaultArg(syzType, ctx)
			}
		}

		return prog.MakeConstArg(syzType, a.Eval(ctx.Target))
	case *parser.DynamicType:
		return prog.MakeConstArg(syzType, a.BeforeCall.Eval(ctx.Target))
	case *parser.GroupType:
		// Sometimes strace represents a pointer to int as [0] which gets parsed
		// as Array([0], len=1). A good example is ioctl(3, FIONBIO, [1]). We may also have an union int type that
		// is a represented as a struct in strace e.g.
		// sigev_value={sival_int=-2123636944, sival_ptr=0x7ffd816bdf30}
		// For now we choose the first option
		if a.Len == 0 {
			log.Fatalf("Parsing const type. Got array type with len 0: %#v", ctx)
		}
		return genConst(syzType, a.Elems[0], ctx)

	case *parser.Field:
		// We have an argument of the form sin_port=IntType(0)
		return genArgs(syzType, a.Val, ctx)
	case *parser.Call:
		// We have likely hit a call like inet_pton, htonl, etc
		return parseInnerCall(syzType, a, ctx)
	case *parser.BufferType:
		// The call almost certainly an error or missing fields
		return genDefaultArg(syzType, ctx)
		// E.g. ltp_bind01 two arguments are empty and
	case *parser.PointerType:
		// This can be triggered by the following:
		// 2435  connect(3, {sa_family=0x2f ,..., 16)
		return prog.MakeConstArg(syzType, a.Address)
	default:
		log.Fatalf("Cannot convert Strace Type: %s to Const Type", traceType.Name())
	}
	return nil
}

func genResource(syzType *prog.ResourceType, traceType parser.IrType, ctx *Context) prog.Arg {
	if syzType.Dir() == prog.DirOut {
		log.Logf(2, "Resource returned by call argument: %s", traceType.String())
		res := prog.MakeResultArg(syzType, nil, syzType.Default())
		ctx.ReturnCache.cache(syzType, traceType, res)
		return res
	}
	switch a := traceType.(type) {
	case parser.Expression:
		val := a.Eval(ctx.Target)
		if arg := ctx.ReturnCache.get(syzType, traceType); arg != nil {
			res := prog.MakeResultArg(syzType, arg.(*prog.ResultArg), syzType.Default())
			return res
		}
		res := prog.MakeResultArg(syzType, nil, val)
		return res
	case *parser.Field:
		return genResource(syzType, a.Val, ctx)
	default:
		log.Fatalf("Resource Type only supports Expression")
	}
	return nil
}

func parseProc(syzType *prog.ProcType, traceType parser.IrType, ctx *Context) prog.Arg {
	if syzType.Dir() == prog.DirOut {
		return genDefaultArg(syzType, ctx)
	}
	switch a := traceType.(type) {
	case parser.Expression:
		val := a.Eval(ctx.Target)
		if val >= syzType.ValuesPerProc {
			return prog.MakeConstArg(syzType, syzType.ValuesPerProc-1)
		}
		return prog.MakeConstArg(syzType, val)
	case *parser.Field:
		return genArgs(syzType, a.Val, ctx)
	case *parser.Call:
		return parseInnerCall(syzType, a, ctx)
	case *parser.BufferType:
		// Again probably an error case
		// Something like the following will trigger this
		// bind(3, {sa_family=AF_INET, sa_data="\xac"}, 3) = -1 EINVAL(Invalid argument)
		return genDefaultArg(syzType, ctx)
	default:
		log.Fatalf("Unsupported Type for Proc: %#v\n", traceType)
	}
	return nil
}

func genDefaultArg(syzType prog.Type, ctx *Context) prog.Arg {
	switch a := syzType.(type) {
	case *prog.PtrType:
		res := prog.DefaultArg(a.Type)
		return addr(ctx, syzType, res.Size(), res)
	case *prog.IntType, *prog.ConstType, *prog.FlagsType, *prog.LenType, *prog.ProcType, *prog.CsumType:
		return prog.DefaultArg(a)
	case *prog.BufferType:
		return prog.DefaultArg(a)
	case *prog.StructType:
		var inner []prog.Arg
		for _, field := range a.Fields {
			inner = append(inner, genDefaultArg(field, ctx))
		}
		return prog.MakeGroupArg(a, inner)
	case *prog.UnionType:
		optType := a.Fields[0]
		return prog.MakeUnionArg(a, genDefaultArg(optType, ctx))
	case *prog.ArrayType:
		return prog.DefaultArg(syzType)
	case *prog.ResourceType:
		return prog.MakeResultArg(syzType, nil, a.Default())
	case *prog.VmaType:
		return prog.DefaultArg(syzType)
	default:
		log.Fatalf("Unsupported Type: %#v", syzType)
	}
	return nil
}

func addr(ctx *Context, syzType prog.Type, size uint64, data prog.Arg) prog.Arg {
	arg := prog.MakePointerArg(syzType, uint64(0), data)
	ctx.Tracker.addAllocation(ctx.CurrentSyzCall, size, arg)
	return arg
}

func reorderStructFields(syzType *prog.StructType, traceType *parser.GroupType, ctx *Context) {
	// Sometimes strace reports struct fields out of order compared to Syzkaller.
	// Example: 5704  bind(3, {sa_family=AF_INET6,
	//				sin6_port=htons(8888),
	//				inet_pton(AF_INET6, "::", &sin6_addr),
	//				sin6_flowinfo=htonl(2206138368),
	//				sin6_scope_id=2049825634}, 128) = 0
	//	The flow_info and pton fields are switched in Syzkaller
	switch syzType.TypeName {
	case "sockaddr_in6":
		log.Logf(5, "Reordering in6")
		field2 := traceType.Elems[2]
		traceType.Elems[2] = traceType.Elems[3]
		traceType.Elems[3] = field2
	case "bpf_insn_generic", "bpf_insn_exit", "bpf_insn_alu", "bpf_insn_jmp", "bpf_insn_ldst":
		log.Logf(2, "bpf_insn_generic size: %d, typsize: %d", syzType.Size(), syzType.TypeSize)
		field1 := traceType.Elems[1].(parser.Expression)
		field2 := traceType.Elems[2].(parser.Expression)
		reg := (field1.Eval(ctx.Target)) | (field2.Eval(ctx.Target) << 4)
		newFields := make([]parser.IrType, len(traceType.Elems)-1)
		newFields[0] = traceType.Elems[0]
		newFields[1] = parser.NewIntsType([]int64{int64(reg)})
		newFields[2] = traceType.Elems[3]
		newFields[3] = traceType.Elems[4]
		traceType.Elems = newFields
	}
}

func shouldSkip(ctx *Context) bool {
	syscall := ctx.CurrentStraceCall
	if utils.ShouldSkip[syscall.CallName] {
		return true
	}
	switch syscall.CallName {
	case "write":
		switch a := syscall.Args[0].(type) {
		case parser.Expression:
			val := a.Eval(ctx.Target)
			if val == 1 || val == 2 {
				return true
			}
		}
	}
	return false
}
