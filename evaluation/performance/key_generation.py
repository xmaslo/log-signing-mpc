import time
import sys
from evaluation.utils.keygen import generate_keys


def keygen_bench(node_count):
    start_time = time.time()

    generate_keys(node_count, 200)

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"Execution time: {execution_time:.2f} seconds")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python key_generation.py <number_of_nodes>")
        sys.exit(1)

    keygen_bench(int(sys.argv[1]))
