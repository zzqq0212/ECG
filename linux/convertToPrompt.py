                            
def fill_prompt(lines,write_path):
    with open(write_path, 'a') as writefile:
        
        isFlag = True
        for index,item in enumerate(lines):
            functions = item.strip("[]").replace("'", "").split(", ")
            if isFlag:
                writefile.write(f"————————————————————————————————These prompts belong to Node: 【{functions[-1]}】at below.—————————————————————————————————— \n\n")
                isFlag = False
            if functions:
                code = f"# The 【{index+1}】th prompt: \n\n"
                code += f"# Now I have a kernel internal function {functions[-1]}, the kernel internal function needs to be invoked from the user mode through the corresponding system call function.\n"
                code += f"## Targeted invocation of kernel internal function: {functions[-1]} in the linux kenrel source code.\n"
                code += f"## The {functions[-1]} function is called in the {functions[0]} system call function in linux kernel.\n"
                code += f"## Detailed linux kernel internal function invocation chain is shown at below.\n"
                    
                for i in range(len(functions) - 1):
                        caller = functions[i]
                        function = functions[i + 1]
                        code += f"{i + 1}. function {caller} invokes {function}.\n"
                code += f"## Based on the show of the kernel internal function chain. please reason step by step. Don't add new kernel module to generate code. Please generate the complete executable C language source code to call the given kernel internal function: {functions[-1]}.\n"
                writefile.write(code + '\n\n')
                
                
def read_txt_file(file_path):
    lines = []
    with open(file_path, 'r') as file:
        for index,line in enumerate(file):
            if line.strip() == "":
                fill_prompt(lines,write_file_path)    
                lines = []
            else:
                lines.append(line.strip())
    if lines:
        fill_prompt(lines, write_file_path)
    
        
if __name__ == "__main__":
    read_file_path = "saved_path_fs_test.txt" 
    write_file_path = "convert_prompt_fs_test.txt"
    read_txt_file(read_file_path)