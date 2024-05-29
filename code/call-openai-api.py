import os
import re
from openai import OpenAI  # type: ignore # 使用 openai 而不是 OpenAI


client = OpenAI(
    api_key = 'sk-HeHttoCiVsRfcpSd3fE5228357574d618aD03e86E23d615d',
    base_url = "https://api.132999.xyz/v1"
)

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
        if prompt:  # Add the last prompt if file does not end with an empty line
            prompts.append(' '.join(prompt).strip())
    return prompts

def call_chatgpt_api(prompt):
    response = client.chat.completions.create(
        model="gpt-4",
        messages=[
            {"role": "user", "content": prompt}
        ],
        max_tokens=1500,
        n=1,
        stop=None,
        temperature=0.7
    )
    return response.choices[0].message.content

def format_response(response):
    # 使用正则表达式找到所有的 ```c``` 块
    code_pattern = re.compile(r'```c\s*(.*?)\s*```', re.DOTALL)
    formatted_response = ""
    last_end = 0
    for match in code_pattern.finditer(response):
        # 非代码块部分
        formatted_response += f"<p>{response[last_end:match.start()].strip()}</p>"
        # 代码块部分
        code = match.group(1)
        formatted_response += f"<pre><code class=\"language-c\">{code.strip()}</code></pre>"
        last_end = match.end()
    # 添加最后一部分
    formatted_response += f"<p>{response[last_end:].strip()}</p>"
    return formatted_response

def generate_output(file_path):
    file_name = os.path.splitext(os.path.basename(file_path))[0]
    output_txt = f"{file_name}_output.txt"
    output_html = f"{file_name}.html"
    output_log = os.path.join("output_log", f"{file_name}_output_response.txt")
    
    if not os.path.exists("output_log"):
        os.makedirs("output_log")

    html_header = """
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
    html_footer = "</body></html>"

    prompts = get_prompts_from_file(file_path)
    with open(output_txt, 'w') as txt_file, open(output_html, 'w') as html_file, open(output_log, 'w') as log_file:
        html_file.write(html_header)  # 先写入HTML头部

        for i, prompt in enumerate(prompts):
            print(f"Processing prompt {i + 1}/{len(prompts)} from file: {file_path}")
            response = call_chatgpt_api(prompt)
            txt_file.write(f"Prompt:\n{prompt}\n\nResponse:\n{response}\n\n{'-'*80}\n\n")
            log_file.write(f"Prompt:\n{prompt}\n\nResponse:\n{response}\n\n{'-'*80}\n\n")
            formatted_response = format_response(response)
            html_content = f"<h3>Prompt:</h3><p>{prompt}</p>"
            html_content += f"<h3>Response:</h3>{formatted_response}<hr>"
            html_file.write(html_content)  # 将内容追加到HTML文件中

        html_file.write(html_footer)  # 最后写入HTML尾部

    print(f"TXT file generated: {output_txt}")
    print(f"HTML file generated: {output_html}")
    print(f"Log file generated: {output_log}")


if __name__ == '__main__':
    txt_files_path = '/mnt/nvme/zhangqiang/LLM-models/prompt'  # 替换为实际的文件夹路径
    txt_files = [os.path.join(txt_files_path, file) for file in os.listdir(txt_files_path) if file.endswith('.txt')]
    
    for txt_file in txt_files:
        generate_output(txt_file)

