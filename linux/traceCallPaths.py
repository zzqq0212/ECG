import pydot

def find_entry_function(dot_file_path):
    entry_function = None
    try:
        graph = pydot.graph_from_dot_file(dot_file_path)
        if graph:
            for node in graph[0].get_node_list():
                nodeName = node.get_name()
                if "do_syscall_64" in nodeName:
                    print("node name: ",nodeName)
                    entry_function = node
                    break
        else:
            print("Error: No graph data found in the DOT file.")
    except Exception as e:
        print(f"An error occurred: {e}")
    return entry_function

dot_file_path = '.dot file path'  # Replace with your .dot file path

entry_function_node = find_entry_function(dot_file_path)

if entry_function_node:
    print("Entry function representing kernel system calls:")
    print(entry_function_node.get_name())
else:
    print("Entry function not found for kernel system calls.")
