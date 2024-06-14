import re
import networkx as nx
import os

def findPathNodesByEndNode(graph):
    system_call_list = []
    node_function_list = []
    pathArray = []
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
        nodePathCount = 0
        for seqindex, item in enumerate(system_call_list):
            if nx.has_path(graph, item.get('node'), node_func):
                paths = nx.all_simple_paths(graph, item.get('node'), node_func)
                if flag:
                    flag = False
                for index, path in enumerate(paths):
                    if index == 0:
                        templength = len(path) - 1
                    pathArray.append(save_path_label(path, node_label_map))
                    nodePathCount = nodePathCount + 1
                    tempcount = tempcount + 1

            if flag != True and seqindex == len(system_call_list) - 1:
                pathArray.append("")
                node_path_count = node_path_count + 1
                flag = True
                nodePathCount = 0

    return pathArray

def save_path_label(path, node_label_map):
    templist = []
    for index, node in enumerate(path):
        if index == 0:
            templist.append(re.sub(r'^__x64_|[0-9]+', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
        else:
            templist.append(re.sub(r'\d+$', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
    return templist

def create_graph_from_dot(dot_file_path):
    print("Loading .dot file")
    graph = nx.DiGraph(nx.drawing.nx_pydot.read_dot(dot_file_path))
    return graph

def fill_prompt(lines):
    prompts = []
    isFlag = True
    for index, item in enumerate(lines):
        functions = item.strip("[]").replace("'", "").split(", ")
        if isFlag:
            prompts.append(f"————————————————————————————————These prompts belong to Node: 【{functions[-1]}】at below.—————————————————————————————————— \n\n")
            isFlag = False
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
            prompts.append(code + '\n\n')
    return prompts

def read_txt_file(pathArray):
    line_groups = []
    current_group = []

    for line in pathArray:
        if not line:
            if current_group:
                line_groups.append(current_group)
                current_group = []
        else:
            current_group.append(str(line).strip())

    if current_group:
        line_groups.append(current_group)

    all_prompts = []
    for group in line_groups:
        prompts = fill_prompt(group)
        all_prompts.extend(prompts)

    return all_prompts

def process_prompts(prompts, file_path):
    new_content = []
    seen_names = set()
    i = 0

    while i < len(prompts):
        line = prompts[i]
        match = re.search(r'These prompts belong to Node: \【(.*?)\】at below.', line)
        if match:
            name = match.group(1)
            if name in seen_names:
                i += 1
                while i < len(prompts) and not prompts[i].startswith('## Based on the show of the kernel internal function chain'):
                    i += 1
                continue
            seen_names.add(name)

        new_content.append(line)
        i += 1

    new_file_path = os.path.splitext(file_path)[0] + '_processed.txt'
    with open(new_file_path, 'w', encoding='utf-8') as file:
        file.writelines(new_content)

if __name__ == "__main__":
    moduleName = 'fs'
    dot_file_path = 'targeted module built.bc.callgraph.dot'  # replace with your actual targetd module built.bc.callgraph.dot file path, such as: linux/built.bc.callgraph.dot

    graph = create_graph_from_dot(dot_file_path)
    pathArray = findPathNodesByEndNode(graph)
    all_prompts = read_txt_file(pathArray)
    process_prompts(all_prompts, f"convert_prompt_{moduleName}.txt")