from evaluation.utils.keygen import generate_keys
import sys

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python key_generation.py <number_of_nodes>")
        sys.exit(1)

    generate_keys(int(sys.argv[1]), 200)
