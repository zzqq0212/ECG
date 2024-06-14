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

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate PC and Distance Mapping")
    parser.add_argument("-i", "--input", required=True, help="Input TXT file with functions and distances")
    parser.add_argument("-j", "--json", required=True, help="Input JSON file with PC and function mapping")
    parser.add_argument("-o", "--output", required=True, help="Output TXT file for PC and distance mapping")
    
    args = parser.parse_args()
    
    # Load the JSON file
    with open(args.json, 'r') as json_file:
        pc_function_mapping = json.load(json_file)
    
    # Read the functions and distances from the input TXT file
    with open(args.input, 'r') as txt_file, open(args.output, 'w') as output_file:
        for line in txt_file:
            parts = line.strip().split(',')
            if len(parts) == 2:
                function = parts[0].strip()
                distance = parts[1].strip()
                weight = dis_to_weight(distance)
                for pc in pc_function_mapping:
                    if pc_function_mapping[pc] == function:
                        output_file.write(f'"0x{pc}":"0x{weight}"\n')



