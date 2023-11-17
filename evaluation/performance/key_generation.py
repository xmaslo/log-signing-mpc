import time
from evaluation.utils.keygen import generate_keys


def keygen_bench(n):
    start_time = time.time()

    generate_keys(n, 200)

    end_time = time.time()
    execution_time = end_time - start_time
    print(f"Execution time: {execution_time:.2f} seconds")


if __name__ == "__main__":
    keygen_bench(3)
