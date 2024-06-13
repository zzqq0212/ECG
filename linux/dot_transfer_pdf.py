from graphviz import Source

dot_file = '.dot file path' # please update the real .dot file path
pdf_file = '6.7-fs.pdf'

with open(dot_file, 'r') as file:
    dot_content = file.read()

src = Source(dot_content)
src.render(pdf_file, view=False, format='pdf')
