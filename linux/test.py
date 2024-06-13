# import re

# # 处理 "__se_sys_semtimedop_time32"
# string = "__se_sys_semtimedop_time32"
# result = re.sub(r'\d*$', '', "__x64se_sys_semtimedop_time32".replace("{", "").replace("}", "").split('.')[0])
# print("去除末尾数字后的结果:", result)


def generate_code(functions):
    code = ""
    code += f"# Now I have a kernel internal function {functions[-1]}, the kernel internal function needs to be invoked from the user mode through the corresponding system call function.\n\n"
    code += f"## Targeted invocation of kernel internal function: {functions[-1]} in the linux kenrel source code.\n\n"
    code += f"## The {functions[-1]} function is called in the {functions[0]} system call function in linux kernel.\n\n"
    code += f"## Detailed linux kernel internal function invocation chain is shown at below.\n\n"
        
    for i in range(len(functions) - 1):
            caller = functions[i]
            function = functions[i + 1]
            code += f"{i + 1}. function {caller} invokes {function}.\n"
    code += "\n"
    code += f"## Based on the show of the kernel internal function chain. please reason step by step. Don't develop kernel module to generate code. Please generate the complete executable C language source code to call the given kernel internal function: {functions[-1]}.\n"
    return code

function_list = ['sys_msgsnd', '__se_sys_msgsnd', '__do_sys_msgsnd', 'ksys_msgsnd', 'do_msgsnd', 'load_msg', '__bad_copy_from']

# Generate code
generated_code = generate_code(function_list)

# Print generated code
print(generated_code)

