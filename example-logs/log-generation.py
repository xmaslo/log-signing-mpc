import json

def parse_json_logs(file_path):
    with open(file_path, 'r') as file:
        for line in file:
            try:
                json_data = json.loads(line.strip())
                # Process the parsed JSON data
                print(json_data)
                # # Retrieve the "time" value from JSON data
                # time = json_data.get("time")
                # if time:
                #     # Process the time value
                #     print(time)
            except json.JSONDecodeError as e:
                print(f"Error parsing JSON: {e}")
                continue

# Example usage
file_path = "nginx_json_logs.txt"
parse_json_logs(file_path)
