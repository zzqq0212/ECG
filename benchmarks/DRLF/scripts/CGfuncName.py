import argparse
import pydotplus as pydot

parser = argparse.ArgumentParser(description='Extract function names from a callgraph .dot file.')
parser.add_argument('-i', '--input', required=True, help='Input callgraph .dot file')
parser.add_argument('-o', '--output', required=True, help='Output text file')
args = parser.parse_args()

graph = pydot.graph_from_dot_file(args.input)

with open(args.output, 'w') as f:
    for node in graph.get_nodes():
        label = node.get_label().strip('"{}')
        function_names = label.split('|')
        for function_name in function_names:
            f.write(function_name.split()[0] + '\n')
