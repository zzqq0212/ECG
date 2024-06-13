import re
import networkx as nx

moduleName = 'net'
system_call_list = []
node_function_list = []
pathArray = []
node_label_map = {}

def findPathNodesByEndNode(graph):
    node_path_count = 1
    flag = True
    pattern = r'.*__x64_sys.*'
    for node, label in nx.get_node_attributes(graph, 'label').items():
        temp = {"node": node, "label": label}
        node_label_map[node] = label
        if re.search(pattern,label):
            system_call_list.append(temp)
        else:
            if len(list(graph.successors(temp.get('node')))) == 0:
                node_function_list.append(temp)
    for func_node_item in node_function_list:
        node_func = func_node_item.get('node')
        node_label = re.sub(r'\d+$', '', node_label_map.get(node_func).replace("\"{", "").replace("}\"", "").split('.')[0])
        tempcount = 1
        nodePathCount = 0
        for seqindex,item in enumerate(system_call_list):
            if nx.has_path(graph,item.get('node'),node_func):
                paths = nx.all_simple_paths(graph,item.get('node'),node_func)
                if flag: 
                    flag = False
                for index,path in enumerate(paths):
                    if index == 0:
                        templength = len(path)-1
                    pathArray.append(save_path_label(path))
                    nodePathCount = nodePathCount + 1
                    tempcount = tempcount + 1
                    
            if flag != True and seqindex == len(system_call_list)-1:
                pathArray.append("")
                node_path_count = node_path_count + 1
                flag = True      
                nodePathCount = 0                           
                    
def save_path_label(path):
    templist = []
    start_label = ""
    end_label = ""
    for index,node in enumerate(path):
        if index == 0:
            start_label = node_label_map.get(node)
        if index == (len(path)-1):
            end_label = node_label_map.get(node)
        if index == 0:
            templist.append(re.sub(r'^__x64_|[0-9]+', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
        else:    
            templist.append(re.sub(r'\d+$', '', node_label_map.get(node).replace("\"{", "").replace("}\"", "").split('.')[0]))
    return templist

def create_graph_from_dot(dot_file_path):
    print("载入 dot 文件开始！")
    with open(dot_file_path, 'r') as file:
        dot_data = file.read()
        graph = nx.DiGraph(nx.drawing.nx_pydot.read_dot(dot_file_path))
        return graph

dot_file_path = '.dot file real path' # replace your actual .dot file path

graph = create_graph_from_dot(dot_file_path)

findPathNodesByEndNode(graph)

if pathArray:
    file_path = 'saved_path_'+moduleName+'.txt'
    with open(file_path, 'w') as file:
        for item in pathArray:
            file.write('%s\n' % item)