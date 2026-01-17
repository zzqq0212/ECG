import os
import re
import argparse
from bs4 import BeautifulSoup

def create_directory(directory_path):
    if not os.path.exists(directory_path):
        os.makedirs(directory_path)

def process_html_file(file_path):
    base_name = os.path.splitext(os.path.basename(file_path))[0]
    base_dir = os.path.join(os.path.dirname(file_path), base_name)
    create_directory(base_dir)

    with open(file_path, 'r', encoding='utf-8') as file:
        soup = BeautifulSoup(file, 'html.parser')

    prompt_elements = soup.find_all('h3', text='Prompt:')

    for idx, prompt in enumerate(prompt_elements, start=1):
        prompt_dir = os.path.join(base_dir, f"Prompt{idx}")
        create_directory(prompt_dir)

        response_element = prompt.find_next('h3', text='Response:')
        if response_element:
            next_prompt = prompt_elements[idx] if idx < len(prompt_elements) else None
            current_element = response_element.find_next()
            
            code_idx = 1
            while current_element and current_element != next_prompt:
                if current_element.name == 'code' and 'language-c' in current_element.get('class', []):
                    c_code = re.sub(r'<[^>]+>', '', str(current_element))
                    c_code = c_code.replace('&lt;', '<').replace('&gt;', '>').replace('&amp;', '&')
                    c_file_name = os.path.join(prompt_dir, f"{code_idx}.c")
                    with open(c_file_name, 'w', encoding='utf-8') as c_file:
                        c_file.write(c_code)
                    code_idx += 1
                current_element = current_element.find_next()

def main():
    parser = argparse.ArgumentParser(description='Process HTML file to extract C code snippets.')
    parser.add_argument('html_file', type=str, help='Path to the HTML file')

    args = parser.parse_args()
    html_file_path = args.html_file

    if not os.path.exists(html_file_path):
        print(f"Error: File {html_file_path} does not exist.")
        return

    process_html_file(html_file_path)

if __name__ == "__main__":
    main()