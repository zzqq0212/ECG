import argparse
import json
import math

def dis_to_weight(distance):
    
    atan_value = math.atan(float(distance))
    normalized_value = (atan_value + math.pi/2) / math.pi 
    value = (1 - normalized_value) * 16 + 1

    hex_value = hex(int(value))[2:]
    weight = hex_value[0]

    return weight

parser = argparse.ArgumentParser(description='Generate PC:Distance mapping from JSON and text files.')
parser.add_argument('-j', '--json-file', required=True, help='JSON file containing PC to function mapping')
parser.add_argument('-i', '--input-file', required=True, help='Text file containing function and distance data')
parser.add_argument('-o', '--output-file', required=True, help='Output file for PC:Distance mapping')
args = parser.parse_args()

pc_func_mapping_file = args.json_file
func_distance_file = args.input_file
output_file = args.output_file

with open(pc_func_mapping_file, 'r') as f:
    pc_func_mapping = json.load(f)

with open(output_file, 'w') as f_out:
    
    with open(func_distance_file, 'r') as f_in:
        
        for line in f_in:
            parts = line.strip().split(',')
            if len(parts) == 2:
                func = parts[0]
                distance = parts[1]

                for pc, func_name in pc_func_mapping.items():
                    if func == func_name:
                        weight = dis_to_weight(distance)
                        f_out.write(f'"0x{pc}:0x{weight}\\n"\n')


print(f"结果已写入到 {output_file} 文件中")
