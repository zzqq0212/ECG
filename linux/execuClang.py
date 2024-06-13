import subprocess

def execute_commands_from_file():
    try:
        with open('build.txt', 'r') as file:
            lines = file.readlines()
            for line in lines:
                command = line.strip()
                if command:  # Check if the line is not empty
                    print(f"Executing command: {command}")
                    subprocess.run(command, shell=True, check=True)
    except FileNotFoundError:
        print("File 'build.txt' not found.")

if __name__ == "__main__":
    execute_commands_from_file()
