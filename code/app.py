from flask import Flask, request, jsonify # type: ignore
import subprocess
import threading
import time

app = Flask(__name__)

def run_ollama_service():
    # 启动 Ollama 服务
    command = ["ollama", "serve"]
    subprocess.Popen(command)
    # 等待服务启动
    time.sleep(10)

def run_mixtral(input_text):
    result = subprocess.run(
        ['ollama', 'run', 'mixtral', input_text],
        capture_output=True,
        text=True
    )
    return result.stdout

@app.route('/api/mixtral', methods=['POST'])
def mixtral_api():
    data = request.json
    if 'text' not in data:
        return jsonify({'error': 'No text provided'}), 400
    input_text = data['text']
    output_text = run_mixtral(input_text)
    return jsonify({'input': input_text, 'output': output_text})

if __name__ == '__main__':
    # 启动 Ollama 服务
    ollama_thread = threading.Thread(target=run_ollama_service)
    ollama_thread.start()
    
    # 启动 Flask 应用
    app.run(host='0.0.0.0', port=5000)
