import networkx as nx
import pydotplus as pydot
import argparse

def Name(label):
    function_name = label.strip('{"}')
    if function_name.count('.') == 1:
        parts = function_name.split('.')
        if parts[1].isdigit():
            function_name = parts[0]

    return function_name


if __name__ == '__main__':

    parser = argparse.ArgumentParser(description='Process a call graph DOT file with input and output options.')
    parser.add_argument('-i', '--input', type=str, required=True, help='Input DOT file name')
    parser.add_argument('-o', '--output', type=str, required=True, help='Output DOT file name')
    args = parser.parse_args()

    input_dot_filename = 'args.input'
    output_dot_filename = args.output
    G = nx.drawing.nx_pydot.read_dot(input_dot_filename)
    callgraph_dot = pydot.Dot(graph_type='digraph')  

    func_to_node = {}

    for node, data in G.nodes(data=True):
        label = data.get('label', '')
        function_name = Name(label)

        if function_name in func_to_node:
            Now_node = func_to_node[function_name]
            
        else:
            Now_node = 'Node' + str(id(node))
            func_to_node[function_name] = Now_node

            now_node_shape = 'record'  #
            now_node_label = '{' + function_name + '}'  

            callgraph_dot.add_node(pydot.Node(Now_node, shape=now_node_shape, label=now_node_label))

        #  添加与之相关的调用边关系
        for neighbor in G.neighbors(node):
            neighbor_label = G.nodes[neighbor].get('label', '')
            neighbor_function_name = Name(neighbor_label)
            neighbor_node_shape = 'record'  
            neighbor_node_label = '{' + neighbor_function_name + '}'  

            if neighbor_function_name in func_to_node:
                neighbor_node = func_to_node[neighbor_function_name]
                
            else:
                neighbor_node = 'Node' + str(id(neighbor))
                func_to_node[neighbor_function_name] = neighbor_node
                callgraph_dot.add_node(pydot.Node(neighbor_node, shape=neighbor_node_shape, label=neighbor_node_label))
            
            callgraph_dot.add_edge(pydot.Edge(Now_node, neighbor_node))

    callgraph_dot.write(output_dot_filename)
