package main

import (
	"bufio"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"
)

var cmd = flag.String("cmd", "kernel", "Build one module or whole kernel, e.g., module, kernel")

var path = flag.String("path", ".", "the path of kernel")

var IsSaveTemps = flag.Bool("isSaveTemp", false, "use -save-temps or -emit-llvm")

var CC = flag.String("CC", "clang", "Name of CC")
var LD = flag.String("LD", "llvm-link", "Name of LD")
var AR = flag.String("AR", "llvm-ar", "Name of AR")
var LLD = flag.String("LLD", "ld.lld", "Name of LD")
var OBJCOPY = flag.String("OBJCOPY", "llvm-objcopy", "Name of OBJCOPY")
var STRIP = flag.String("STRIP", "llvm-strip", "Name of STRIP")

var ToolChain = flag.String("toolchain", "", "Path of clang and llvm-link")

var FlagCC = FlagAll + FlagCCNoNumber

const (
	PrefixCmd  = "savedcmd_"
	PrefixCmd2 = "cmd_"
	SuffixCmd  = ".cmd"
	SuffixCC   = ".o.cmd"

	SuffixLD   = ".a.cmd"
	SuffixLTO  = ".lto.o.cmd"
	SuffixKO   = ".ko.cmd"
	NameScript = "extractbc.sh"

	NameClang = "clang"

	FlagAll = " -w -g"

	FlagCCNoOptzns = " -mllvm -disable-llvm-optzns"

	FlagCCNoNumber = " -fno-discard-value-names"

	NameLD    = "llvm-link"
	FlagLD    = " -v "
	FlagOutLD = " -o "

	CmdLinkVmlinux = " -v -o cg-build.bc"

	CmdTools = "BUILD_STR(s)=$(pound)s"
)

var bitcodes map[string]bool
var linkedBitcodes map[string]bool
var builtinModules map[string]bool

func getCmd(cmdFilePath string) string {
	res := ""
	if _, err := os.Stat(cmdFilePath); os.IsNotExist(err) {
		fmt.Printf(cmdFilePath + " does not exist\n")
	} else {
		file, err := os.Open(cmdFilePath)
		if err != nil {
			log.Fatal(err)
		}
		defer func(file *os.File) {
			err := file.Close()
			if err != nil {

			}
		}(file)

		scanner := bufio.NewScanner(file)
		scanner.Split(bufio.ScanLines)

		var text []string
		for scanner.Scan() {
			text = append(text, scanner.Text())
		}
		for _, eachLine := range text {
			if strings.HasPrefix(eachLine, PrefixCmd) || strings.HasPrefix(eachLine, PrefixCmd2) {
				i := strings.Index(eachLine, ":=")
				if i > -1 {
					cmd := eachLine[i+3:]
					res = cmd
					break
				} else {
					fmt.Println("Cmd Index not found")
					fmt.Println(eachLine)
				}
			}
		}
		if err := scanner.Err(); err != nil {
			log.Fatal(err)
		}
	}
	res += "\n"
	res = res[strings.Index(res, ""):]
	return res
}

func handleCC(cmd string) string {
	res := ""
	if i := strings.Index(cmd, " -c "); i > -1 {

		if j := strings.Index(cmd, CmdTools); j > -1 {
			return res
		}

		res += cmd[:i]
		res += FlagCC
		if *IsSaveTemps {
			res += " -save-temps=obj"
		} else {
			res += " -emit-llvm"
		}
		res += cmd[i:]

		if *IsSaveTemps {

		} else {
			res = strings.Replace(res, ".o ", ".bc ", -1)
		}

		if strings.Count(res, " ; ") == 1 {
			i := strings.Index(res, ";")
			res = res[:i] + "\n"
		}
		res = strings.TrimSpace(res) + "\n"

		if strings.HasSuffix(res, ".S\n") {
			s1 := strings.Split(res, " ")
			s2 := s1[len(s1)-2]
			s4 := strings.Replace(s2, ".o ", ".bc ", -1)
			res = "echo \"\" > " + s4 + "\n"
		}
	} else {
		fmt.Println("CC Index not found")
		fmt.Println(cmd)
	}
	res = " " + res
	res = strings.Replace(res, " "+*CC+" ", " "+filepath.Join(*ToolChain, NameClang)+" ", -1)
	res = strings.Replace(res, " -Os ", " -O0 ", -1)
	res = strings.Replace(res, " -O3 ", " -O0 ", -1)
	res = strings.Replace(res, " -O2 ", " -O0 ", -1)
	res = strings.Replace(res, " -fno-var-tracking-assignments ", "  ", -1)
	res = strings.Replace(res, " -fconserve-stack ", "  ", -1)
	res = strings.Replace(res, " -march=armv8-a+crypto ", "  ", -1)
	res = strings.Replace(res, " -mno-fp-ret-in-387 ", "  ", -1)
	res = strings.Replace(res, " -mskip-rax-setup ", "  ", -1)
	res = strings.Replace(res, " -ftrivial-auto-var-init=zero ", "  ", -1)

	return res
}

func handleSuffixCCWithLD(cmd string, path string) string {

	if strings.Count(cmd, " ; ") == 1 {
		i := strings.Index(cmd, ";")
		cmd = cmd[:i] + "\n"
	}
	cmd = strings.TrimSpace(cmd) + "\n"

	res := ""
	if strings.Index(cmd, "@") > -1 {
		fileName := cmd[strings.Index(cmd, "@")+1 : len(cmd)-1]
		filePath := filepath.Join(path, fileName)
		file, err := os.Open(filePath)
		if err != nil {
			log.Println("handleSuffixCCWithLD file error: ")
			log.Fatal(err)
		}
		defer func(file *os.File) {
			err := file.Close()
			if err != nil {

			}
		}(file)

		scanner := bufio.NewScanner(file)
		scanner.Split(bufio.ScanLines)

		var text []string
		for scanner.Scan() {
			text = append(text, scanner.Text())
		}

		if err := scanner.Err(); err != nil {
			log.Fatal(err)
		}

		res += filepath.Join(*ToolChain, NameLD)
		res += FlagLD
		res += FlagOutLD
		res += cmd[strings.Index(cmd, FlagOutLD)+len(FlagOutLD) : strings.Index(cmd, "@")]

		for _, s := range text {
			res += s + " "
		}

		res = strings.Replace(res, ".o ", ".bc ", -1)
		res += "\n"

	} else if strings.HasPrefix(cmd, *LLD) {
		res += filepath.Join(*ToolChain, NameLD)
		res += FlagLD
		res += FlagOutLD

		cmd = cmd[:len(cmd)-1]
		s1 := strings.Split(cmd, " ")
		obj := ""
		for _, s := range s1 {
			if strings.HasSuffix(s, ".o") {
				obj = " " + strings.Replace(s, ".o", ".bc", -1) + obj
			}
		}
		res += obj
		res += "\n"
	} else {
		fmt.Println("handleSuffixCCWithLD cmd error: " + cmd)
	}
	return res
}

func handleOBJCOPY(cmd string) string {
	res := filepath.Join(*ToolChain, NameLD) + FlagLD + FlagOutLD
	cmd = cmd[:len(cmd)-1]
	s1 := strings.Split(cmd, " ")
	obj := ""
	for _, s := range s1 {
		if strings.HasSuffix(s, ".o") {
			obj = " " + strings.Replace(s, ".o", ".bc", -1) + obj
		}
	}
	res += obj
	res += "\n"
	return res
}

func handleSTRIP(cmd string) string {
	res := filepath.Join(*ToolChain, NameLD) + FlagLD + FlagOutLD
	s1 := strings.Split(cmd, ";")
	cmd = s1[0]
	s1 = strings.Split(cmd, " ")
	for _, s := range s1 {
		if strings.HasSuffix(s, ".o") {
			res = res + " " + strings.Replace(s, ".o", ".bc", -1)
		}
	}
	res += "\n"
	return res
}

func handleLD(cmd string) string {
	replace := func(cmd string) string {
		res := ""
		if strings.Count(cmd, ".") > 1 {
			res = cmd
			res = strings.Replace(res, " "+*AR+" ", " "+filepath.Join(*ToolChain, NameLD)+FlagLD+FlagOutLD, -1)
		} else {
			res = "echo \"\" > " + cmd
			res = strings.Replace(res, " "+*AR+" ", " ", -1)
		}

		res = strings.Replace(res, " rcSTPD ", " ", -1)
		res = strings.Replace(res, " cDPrST ", " ", -1)
		res = strings.Replace(res, " cDPrsT ", " ", -1)
		res = strings.Replace(res, " rcsD ", " ", -1)

		res = strings.Replace(res, ".o", ".bc", -1)
		res = strings.Replace(res, ".a ", ".bc ", -1)
		res = strings.Replace(res, ".a\n", ".bc\n", -1)

		// for this drivers/misc/lkdtm/rodata.bc
		res = strings.Replace(res, "rodata_objcopy.bc", "rodata.bc", -1)
		res = strings.Replace(res, " drivers/of/unittest-data/cg-build.bc", "", -1)

		return res
	}

	res := ""
	cmds := strings.Split(cmd, "; ")
	for _, cmd := range cmds {
		if i := strings.Index(cmd, " "+*AR+" "); i > -1 {
			res += replace(cmd)
		}
	}
	return res
}

func handleLTO(cmd string) string {
	res := ""
	res += filepath.Join(*ToolChain, NameLD)
	res += FlagLD
	res += FlagOutLD

	cmd = cmd[strings.Index(cmd, FlagOutLD) : len(cmd)-1]
	objs := strings.Split(cmd, " ")
	output := false
	for _, obj := range objs {
		if obj == "-o" {
			output = true
		} else if output && obj != "" {
			res += strings.Replace(obj, ".o", ".bc", -1)
			output = false
		} else if strings.HasSuffix(obj, ".o") {
			res += " " + strings.Replace(obj, ".o", ".bc", -1)
		}
	}
	res += "\n"
	return res
}

func handleKO(cmd string) (string, string) {
	res := ""
	res += filepath.Join(*ToolChain, NameLD)
	res += FlagLD
	res += FlagOutLD

	cmd = cmd[strings.Index(cmd, FlagOutLD)+len(FlagOutLD):]
	cmd = strings.Replace(cmd, ".ko", ".ko.bc", -1)
	cmd = strings.Replace(cmd, ".o", ".bc", -1)

	if strings.Count(cmd, ";") >= 1 {
		i := strings.Index(cmd, ";")
		cmd = cmd[:i] + "\n"
	}

	moduleFile := cmd[:strings.Index(cmd, ".ko.bc")+len(".ko.bc")]
	res += cmd

	return res, moduleFile
}

func build(kernelPath string) (string, string) {
	cmdCC := ""
	cmdLDInCC := ""

	err := filepath.Walk(kernelPath,
		func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if strings.HasSuffix(info.Name(), SuffixCC) && !strings.HasSuffix(info.Name(), SuffixLTO) {
				cmd := getCmd(path)
				if strings.HasPrefix(cmd, *CC) {
					cmd := handleCC(cmd)
					cmdCC += cmd
				} else if strings.Index(cmd, *AR) > -1 {
					cmd = handleLD(cmd)
					cmdLDInCC = cmd + cmdLDInCC
				} else if strings.Index(cmd, *LLD) > -1 {
					cmd = handleSuffixCCWithLD(cmd, kernelPath)
					cmdLDInCC = cmd + cmdLDInCC
				} else if strings.HasPrefix(cmd, *OBJCOPY) {
					cmd = handleOBJCOPY(cmd)
					cmdLDInCC = cmd + cmdLDInCC
				} else if strings.HasPrefix(cmd, *STRIP) {
					cmd = handleSTRIP(cmd)
					cmdLDInCC = cmd + cmdLDInCC
				} else {
					fmt.Println(*CC + " not found")
					fmt.Println(path)
					fmt.Println(cmd)
				}
			}
			return nil
		})
	if err != nil {
		log.Println(err)
	}

	cmdLink := ""
	moduleFiles := ""
	err = filepath.Walk(kernelPath,
		func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if strings.HasSuffix(info.Name(), SuffixLD) {
				cmd := getCmd(path)
				cmd = handleLD(cmd)
				cmdLink = cmd + cmdLink
				if strings.Index(cmd, FlagOutLD) > -1 {
					obj := cmd[strings.Index(cmd, FlagOutLD)+len(FlagOutLD) : len(cmd)-1]
					if _, ok := linkedBitcodes[obj]; ok {

					} else {
						if obj == "cg-build.bc" {

						} else {
							builtinModules[obj] = true
						}

					}

					objs := strings.Split(cmd[:strings.Index(cmd, "|")], " ")
					base := ""
					for _, bc := range objs {
						if i := strings.Index(bc, "%s"); i > -1 {
							base = bc[1:strings.Index(bc, "%s")]
							if base == "./" {
								base = ""
							}
						}
						if strings.Index(bc, ".bc") > -1 {
							linkedBitcodes[base+bc] = true
						}
					}
				}

			} else if strings.HasSuffix(info.Name(), SuffixLTO) {
				cmd := getCmd(path)
				cmdLink = handleLTO(cmd) + cmdLink

			} else if strings.HasSuffix(info.Name(), SuffixKO) {
				cmd, moduleFile := handleKO(getCmd(path))
				cmdLink = cmd + cmdLink
				moduleFiles = moduleFile + " " + moduleFiles
			}

			return nil
		})

	if err != nil {
		log.Println(err)
	}

	fmt.Println("moduleFiles: ")
	fmt.Println(moduleFiles)

	var resFinal string
	for module := range builtinModules {
		resFinal += " " + module
	}

	return cmdCC + cmdLDInCC + cmdLink + "\n# external modules: " + moduleFiles + "\n", resFinal
}

func generateScript(path string, cmd string) {
	res := "#!/bin/bash\n"
	res += cmd

	pathScript := filepath.Join(NameScript)
	_ = os.RemoveAll(pathScript)
	fmt.Printf("script path : bash %s\n", pathScript)
	f, err := os.OpenFile(pathScript, os.O_WRONLY|os.O_CREATE, 0666)
	if err != nil {
		log.Println(err)
	}
	defer func(f *os.File) {
		err := f.Close()
		if err != nil {

		}
	}(f)

	_, _ = f.WriteString(res)
	_, _ = f.WriteString("\n# path: " + path + "\n")
}

func main() {
	flag.Parse()
	*path, _ = filepath.Abs(*path)

	bitcodes = make(map[string]bool)
	linkedBitcodes = make(map[string]bool)
	builtinModules = make(map[string]bool)

	switch *cmd {
	case "module":
		{
			fmt.Printf("Build module\n")
			res, _ := build(*path)
			generateScript(*path, res)
		}
	case "kernel":
		{
			fmt.Printf("Build kernel and external module\n")
			res, res5 := build(*path)
			if res5 != "" {
				res += filepath.Join(*ToolChain, NameLD) + CmdLinkVmlinux + res5 + "\n"
			}
			generateScript(*path, res)
		}
	default:
		fmt.Printf("cmd is invalid\n")
	}
}
