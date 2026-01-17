import re
import networkx as nx
import os
from io import StringIO

def find_path_nodes_by_end_node(graph):
    system_call_list = []
    node_function_list = []
    path_array = []
    node_label_map = {}
    node_path_count = 1
    flag = True
    pattern = r'.*__x64_sys.*'
    
    for node, label in nx.get_node_attributes(graph, 'label').items():
        temp = {"node": node, "label": label}
        node_label_map[node] = label
        if re.search(pattern, label):
            system_call_list.append(temp)
        else:
            if len(list(graph.successors(temp.get('node')))) == 0:
                node_function_list.append(temp)
    
    for func_node_item in node_function_list:
        node_func = func_node_item.get('node')
        node_label = re.sub(r'\d+$', '', node_label_map.get(node_func).replace("\"{", "").replace("}\"", "").split('.')[0])
        tempcount = 1
        node_path_count = 0
        for seqindex, item in enumerate(system_call_list):
            if nx.has_path(graph, item.get('node'), node_func):
                paths = nx.all_simple_paths(graph, item.get('node'), node_func)
                if flag:
                    flag = False
                for index, path in enumerate(paths):
                    if index == 0:
                        templength = len(path) - 1
                    path_array.append(save_path_label(path, node_label_map))
                    node_path_count += 1
                    tempcount += 1
            if not flag and seqindex == len(system_call_list) - 1:
                path_array.append("")
                node_path_count += 1
                flag = True
                node_path_count = 0
    return path_array

def save_path_label(path, node_label_map):
    templist = []
    for index, node in enumerate(path):
        if index == 0:
            templist.append(re.sub(r'^__x64_|[0-9]+', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
        else:
            templist.append(re.sub(r'\d+$', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
    return templist

def create_graph_from_dot(dot_data):
    graph = nx.DiGraph(nx.drawing.nx_pydot.read_dot(StringIO(dot_data)))
    return graph

def fill_prompt(lines):
    output = StringIO()
    is_flag = True
    for index, item in enumerate(lines):
        functions = item.strip("[]").replace("'", "").split(", ")
        if is_flag:
            output.write(f"————————————————————————————————These prompts belong to Node: 【{functions[-1]}】at below.—————————————————————————————————— \n\n")
            is_flag = False
        if functions:
            code = f"# The 【{index + 1}】th prompt: \n\n"
            code += f"# Now I have a kernel internal function {functions[-1]}, the kernel internal function needs to be invoked from the user mode through the corresponding system call function.\n"
            code += f"## Targeted invocation of kernel internal function: {functions[-1]} in the linux kenrel source code.\n"
            code += f"## The {functions[-1]} function is called in the {functions[0]} system call function in linux kernel.\n"
            code += f"## Detailed linux kernel internal function invocation chain is shown at below.\n"
            for i in range(len(functions) - 1):
                caller = functions[i]
                function = functions[i + 1]
                code += f"{i + 1}. function {caller} invokes {function}.\n"
            code += f"## Based on the show of the kernel internal function chain. please reason step by step. Don't add new kernel module to generate code. Please generate the complete executable C language source code to call the given kernel internal function: {functions[-1]}.\n"
            output.write(code + '\n\n')
    return output.getvalue()

def process_saved_paths(saved_paths_content):
    lines = saved_paths_content.strip().split('\n')
    result = []
    buffer = []
    for line in lines:
        if line.strip() == "":
            if buffer:
                result.append(fill_prompt(buffer))
                buffer = []
        else:
            buffer.append(line.strip())
    if buffer:
        result.append(fill_prompt(buffer))
    return ''.join(result)

def process_final_content(content):
    lines = content.split('\n')
    new_content = []
    i = 0
    seen_names = set()
    skip_section = False

    while i < len(lines):
        line = lines[i]
        match = re.search(r'These prompts belong to Node: \【(.*?)\】at below.', line)
        if match:
            name = match.group(1)
            if 'kasan' in name or 'kcsan' in name:
                skip_section = True

        if skip_section:
            if 'These prompts belong to Node' in line and not ('kasan' in line or 'kcsan' in line):
                skip_section = False
            else:
                i += 1
                continue

        if match:
            name = match.group(1)
            if name in seen_names:
                i += 1
                while i < len(lines) and not lines[i].startswith('## Based on the show of the kernel internal function chain'):
                    i += 1
                continue
            seen_names.add(name)

        if 'These prompts belong to Node' in line:
            new_content.append('\n' + line)
            i += 1
            continue

        if '# The 【1】th prompt:' in line:
            i += 1
            while i < len(lines) and not lines[i].startswith('# Now I have a kernel internal function'):
                i += 1
            while i < len(lines):
                sub_line = lines[i]
                new_content.append(sub_line)
                if sub_line.startswith('## Based on the show of the kernel internal function chain'):
                    i += 1
                    break
                i += 1
            continue
        
        i += 1

    return '\n'.join(new_content)

if __name__ == "__main__":
    dot_file_path = 'path' # replace .dot file actual path  /path/to/your/xxx.dot
    
    with open(dot_file_path, 'r') as file:
        dot_file_content = file.read()
    graph = create_graph_from_dot(dot_file_content)
    path_array = find_path_nodes_by_end_node(graph)
    saved_paths_content = '\n'.join(map(str, path_array))
    prompts_content = process_saved_paths(saved_paths_content)
    final_content = process_final_content(prompts_content)
    with open("convert_prompt_sched_processed.txt", "w", encoding='utf-8') as final_file:
        final_file.write(final_content)