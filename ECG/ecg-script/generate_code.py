import requests
import os
import re

def get_prompts_from_file(file_path):
    prompts = []
    with open(file_path, 'r') as file:
        prompt = []
        for line in file:
            line = line.strip()
            if "These prompts belong to Node:" in line:
                continue
            if line == "":
                if prompt:
                    prompts.append(' '.join(prompt).strip())
                    prompt = []
            else:
                prompt.append(line)
        if prompt:
            prompts.append(' '.join(prompt).strip())
    return prompts

def call_mixtral_api(prompt):
    url = "http://localhost:5000/api/mixtral"
    headers = {'Content-Type': 'application/json'}
    payload = {'text': prompt}
    response = requests.post(url, headers=headers, json=payload)
    if response.status_code == 200:
        return response.json().get('output', '')
    else:
        return f"Error: {response.status_code}"

def format_response(response):
    code_pattern = re.compile(r'```(\w+)\s*(.*?)\s*```', re.DOTALL)
    formatted_response = ""
    last_end = 0
    for match in code_pattern.finditer(response):
        formatted_response += f"<p>{response[last_end:match.start()].strip()}</p>"
        language = match.group(1)
        code = match.group(2).replace('<', '&lt;').replace('>', '&gt;')
        formatted_response += f"<pre><code class=\"language-{language}\">{code.strip()}</code></pre>"
        last_end = match.end()
    formatted_response += f"<p>{response[last_end:].strip()}</p>"
    return formatted_response

def generate_html(file_path):
    file_name = os.path.splitext(os.path.basename(file_path))[0]
    output_html = f"{file_name}.html"
    html_content = """
    <html>
    <head>
        <style>
            body { font-family: Arial, sans-serif; }
            h3 { color: #333; }
            p { margin: 0; padding: 0.5em 0; }
            pre { background-color: #f4f4f4; padding: 1em; border-radius: 5px; overflow-x: auto; }
            code { font-family: 'Courier New', Courier, monospace; }
        </style>
    </head>
    <body>
    """
    
    prompts = get_prompts_from_file(file_path)
    for prompt in prompts:
        response = call_mixtral_api(prompt)
        formatted_response = format_response(response)
        html_content += f"<h3>Prompt:</h3><p>{prompt}</p>"
        html_content += f"<h3>Response:</h3>{formatted_response}<hr>"
    
    html_content += "</body></html>"

    with open(output_html, 'w') as file:
        file.write(html_content)
    
    print(f"HTML file generated: {output_html}")

if __name__ == '__main__':
    txt_files_path = 'path'  # # please update your actual file folder path
    txt_files = [os.path.join(txt_files_path, file) for file in os.listdir(txt_files_path) if file.endswith('.txt')]
    
    for txt_file in txt_files:
        generate_html(txt_file)
