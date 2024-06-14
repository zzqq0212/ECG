import argparse
import subprocess
import math
import json
from multiprocessing import Process, Queue
from glob import glob

# 创建命令行参数解析器
parser = argparse.ArgumentParser(description='将PC地址映射到源代码位置并合并到一个JSON文件中')
parser.add_argument('-i', '--input', type=str, required=True, help='输入文件名')
parser.add_argument('-o', '--output', type=str, required=True, help='输出文件名 (JSON)')
parser.add_argument('-e', '--vmlinux', type=str, required=True, help='vmlinux 文件路径')
args = parser.parse_args()

# 输入文件路径
unique_pcs_file = args.input
vmlinux_path = args.vmlinux  # 从命令行参数获取 vmlinux 路径

def process_batch(batch_id, total_batches, result_queue):
    pc_results = []
    with open(unique_pcs_file, 'r') as infile:
        lines = infile.readlines()
        batch_size = math.ceil(len(lines) / total_batches)
        start_line = batch_id * batch_size
        end_line = min((batch_id + 1) * batch_size, len(lines))
        lines_to_process = lines[start_line:end_line]
        for pc in lines_to_process:
            pc = pc.strip()
            try:
                result = subprocess.run(['addr2line', '-fi', '-e', vmlinux_path, pc], capture_output=True, text=True, check=True)
                output = result.stdout.strip()
                pc_result = {
                    'pc': pc,
                    'func': output.splitlines()[0],
                    'file': output.splitlines()[1]
                }
                pc_results.append(pc_result)
            except subprocess.CalledProcessError as e:
                pc_result = {
                    'pc': pc,
                    'func': 'Error',
                    'file': e.stderr.strip()
                }
                pc_results.append(pc_result)

    result_queue.put(pc_results)

if __name__ == '__main__':
    total_batches = 16
    processes = []
    result_queue = Queue()

    for i in range(total_batches):
        p = Process(target=process_batch, args=(i, total_batches, result_queue))
        processes.append(p)
        p.start()

    for p in processes:
        p.join()

    all_pc_results = []
    for _ in range(total_batches):
        pc_results = result_queue.get()
        all_pc_results.extend(pc_results)

    output_file = args.output
    with open(output_file, 'w') as json_file:
        json.dump(all_pc_results, json_file, indent=2)

    print("已完成所有任务，结果已保存到 JSON 文件：", output_file)
