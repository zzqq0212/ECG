import os
import re

def process_txt_file(file_path):
    with open(file_path, 'r', encoding='utf-8') as file:
        lines = file.readlines()

    new_content = []
    i = 0
    seen_names = set()
    skip_section = False

    while i < len(lines):
        line = lines[i]
        match = re.search(r'These prompts belong to Node: \【(.*?)\】at below.', line)
        if match:
            name = match.group(1)
            if 'kasan' in name or 'kcsan' in name:
                skip_section = True

        if skip_section:
            if 'These prompts belong to Node' in line and not ('kasan' in line or 'kcsan' in line):
                skip_section = False
            else:
                i += 1
                continue

        if match:
            name = match.group(1)
            if name in seen_names:
                i += 1
                while i < len(lines) and not lines[i].startswith('## Based on the show of the kernel internal function chain'):
                    i += 1
                continue
            seen_names.add(name)

        if 'These prompts belong to Node' in line:
            new_content.append('\n' + line)
            i += 1
            continue

        if '# The 【1】th prompt:' in line:
            i += 1

            while i < len(lines) and not lines[i].startswith('# Now I have a kernel internal function'):
                i += 1
            
            while i < len(lines):
                sub_line = lines[i]
                new_content.append(sub_line)
                if sub_line.startswith('## Based on the show of the kernel internal function chain'):
                    i += 1
                    break
                i += 1
            continue
        
        i += 1

    new_file_path = os.path.splitext(file_path)[0] + '_processed.txt'
    with open(new_file_path, 'w', encoding='utf-8') as file:
        file.writelines(new_content)

def process_all_txt_files(directory_path):
    for file_name in os.listdir(directory_path):
        if file_name.endswith('.txt'):
            file_path = os.path.join(directory_path, file_name)
            process_txt_file(file_path)

if __name__ == "__main__":
    directory_path = '.' 
    process_all_txt_files(directory_path)
