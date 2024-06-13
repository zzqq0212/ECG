import pydotplus

def visualize_dot_file(dot_file_path, output_pdf_path, dpi=300):
    try:
        graph = pydotplus.graph_from_dot_file(dot_file_path)
        if graph:
            graph.set_dpi(dpi)
            graph.write_pdf(output_pdf_path)
            print(f"High-definition PDF '{output_pdf_path}' generated successfully.")
        else:
            print("Error: No graph data found in the DOT file.")
    except Exception as e:
        print(f"An error occurred: {e}")

dot_file_path = '.dot file path'  # Replace with your .dot file path
output_pdf_path = '6.7-fs-cg.pdf'   

visualize_dot_file(dot_file_path, output_pdf_path)
