import time
import sys
from evaluation.utils.keygen import generate_keys
import os


def remove_json_and_local_share_files(directory_path):
    try:
        files = os.listdir(directory_path)
 
        for file in files:
            if file.endswith(".json") and file.startswith("local-share"):
                file_path = os.path.join(directory_path, file)
                os.remove(file_path)
 
    except Exception as e:
        print(f"An error occurred: {e}")


def keygen_bench(node_count):
    start_time = time.time()

    generate_keys(node_count, 200)

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"Execution time: {execution_time:.2f} seconds")
    return execution_time


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python key_generation.py <number_of_nodes> <number_of_trials> <path_to_generated_keys>")
        sys.exit(1)

    number_of_nodes = int(sys.argv[1])
    number_of_trials = int(sys.argv[2])
    path_to_keys = sys.argv[3]
    
    execution_times = []
    for i in range(number_of_trials):
        if path_to_keys != "none":
            remove_json_and_local_share_files(path_to_keys)
        execution_times.append(keygen_bench(int(sys.argv[1])))
    
    print(f"Times: {execution_times}")
    print(f"Average: {sum(execution_times)/len(execution_times)}")
