import argparse
import subprocess
import json

parser = argparse.ArgumentParser(description="Process PC addresses using addr2line.")
parser.add_argument("-i", "--input", required=True, help="Input file containing PC addresses.")
parser.add_argument("-ofunc", "--output_func", required=True, help="Output JSON file for storing functions.")
parser.add_argument("-ofile", "--output_file", required=True, help="Output JSON file for storing files.")
parser.add_argument("-e", "--vmlinux", required=True, help="Path to the vmlinux file.")
args = parser.parse_args()

pc_func_mapping = {}  
pc_file_mapping = {} 

with open(args.input, "r") as infile:
    for line in infile:
        pc_address = line.strip()
        result = subprocess.check_output(["addr2line", "-f", "-e", args.vmlinux, pc_address], universal_newlines=True)
        output_lines = result.strip().split('\n')  
        
        if len(output_lines) >= 2:
            function = output_lines[0]
            file = output_lines[1]
            pc_func_mapping[pc_address] = function
            pc_file_mapping[pc_address] = file

with open(args.output_func, "w") as func_json:
    json.dump(pc_func_mapping, func_json, indent=4)

with open(args.output_file, "w") as file_json:
    json.dump(pc_file_mapping, file_json, indent=4)
