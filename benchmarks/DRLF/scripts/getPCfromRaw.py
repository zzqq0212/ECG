import argparse

parser = argparse.ArgumentParser(description='"<__sanitizer_cov_trace_pc>"PC')
parser.add_argument('-i', '--input', type=str, required=True, help='input_file_name')
parser.add_argument('-o', '--output', type=str, required=True, help='output_file_name')
args = parser.parse_args()

input_file = args.input
output_file = args.output

with open(input_file, 'r') as infile, open(output_file, 'w') as outfile:
    for line in infile:
        if "<__sanitizer_cov_trace_pc>" in line:
            pc_value = line.split(':')[0].strip()
            outfile.write(pc_value + '\n')


