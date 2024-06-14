import argparse
import collections
import functools
import pydotplus as pydot
import networkx as nx
import subprocess


class memoize:
    # From https://github.com/S2E/s2e-env/blob/master/s2e_env/utils/memoize.py

    def __init__(self, func):
        self._func = func
        self._cache = {}

    def __call__(self, *args):
        if not isinstance(args, collections.abc.Hashable):
            return self._func(args)

        if args in self._cache:
            return self._cache[args]

        value = self._func(*args)
        self._cache[args] = value
        return value

    def __repr__(self):
        # Return the function's docstring
        return self._func.__doc__

    def __get__(self, obj, objtype):
        # Support instance methods
        return functools.partial(self.__call__, obj)

def node_name(name):
    return "\"{%s}\"" % name

@memoize
def find_nodes(name):
    n_name = node_name(name)
    return [n for n, d in G.nodes(data=True) if n_name in d.get('label', '')]

def distance(name):
    distance = -1
    for n in find_nodes(name):
        d = 0.0
        i = 0

        for t in targets:
            try:
                shortest = nx.dijkstra_path_length(G, n, t)
                d += 1.0 / (1.0 + shortest)
                i += 1
            except nx.NetworkXNoPath:
                pass

        if d != 0 and (distance == -1 or distance > i / d):
            distance = i / d

    if distance != -1:
        out.write(name)
        out.write(",")
        out.write(str(distance))
        out.write("\n")
    
    return distance

# pc -> func
def pc_to_func(pc, kernel_path):
    cmd = f"addr2line -f -e {kernel_path} {pc}"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    output_lines = result.stdout.strip().split('\n')
    func_name = output_lines[0]
    return func_name

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('-d', '--dot', type=str, required=True, help="Path to dot-file representing the graph.")
    parser.add_argument('-t', '--targets', type=str, required=True, help="Path to file specifying Target nodes.")
    parser.add_argument('-o', '--out', type=str, required=True, help="Path to output file containing distance for each node.")
    parser.add_argument('-k', '--kernel', type=str, required=True, help="Path to kernel vmlinux file.")
    parser.add_argument('-p', '--pcs', type=str, help="Path to pcs.txt containing PC addresses.")
    parser.add_argument('-n', '--names', type=str, help="Path to file containing name for each node.")

    args = parser.parse_args()

    print("\nParsing %s .." % args.dot)
    G = nx.drawing.nx_pydot.read_dot(args.dot)
    print("Node count:", G.number_of_nodes())
    print("Edge count:", G.number_of_edges())

    # loading targets func 
    print("Loading targets..")

    with open(args.targets, "r") as f:
        targets = []
        for line in f.readlines():
            line = line.strip()
            for target in find_nodes(line):
                targets.append(target)

    if not targets:
        print("No targets available")
        exit(0)

    distances = []  # store distance

    print("Calculating distance..")

    if args.names:
        with open(args.out, "w") as out, open(args.names, "r") as f:
            for line in f.readlines():
                dist = distance(line.strip())
                if dist != -1:
                    distances.append(dist)
    
    elif args.pcs:  
        with open(args.out, "w") as out, open(args.pcs, 'r') as f:
            for pc in f.readlines():

                print("now calculate pc is: ")
                print(pc)
                func_name = pc_to_func(pc, args.kernel)
                dist = distance(func_name)
                if dist != -1:
                    distances.append(dist)
                #
                print(dist)
       
    # calculte weight
    if distances and len(distances) > 1:
        min_distance = min(distances)
        max_distance = max(distances)
        avg_distance = sum(distances) / len(distances)
        weight = (avg_distance - min_distance) / (max_distance - min_distance)
        with open(args.out, "w") as out:
            out.write(str(weight))
        print("weight: ")
        print(weight)
    
    else:
        print("No weight calculation possible due to insufficient distances")



