IS_DOCKER = False

BASE_URL = "http://localhost"

SERVER_PORT0 = "8001"
SERVER_PORT1 = "8002"
SERVER_PORT2 = "8003"

if IS_DOCKER:
    URL0 = "la1:3001"
    URL1 = "la2:3002"
    URL2 = "la3:3003"
else:
    URL0 = "127.0.0.1:3001"
    URL1 = "127.0.0.1:3002"
    URL2 = "127.0.0.1:3003"


def get_urls(n, is_docker):
    urls = []
    for i in range(1, n + 1):
        if is_docker:
            urls.append(f"la{i}:800{i}")
        else:
            urls.append(f"http://127.0.0.1:800{i}")

    return urls


def get_keygen_payloads(n, is_docker):
    payloads = []
    for i in range(n):
        payload = ""
        for j in range(n):
            if i != j:
                if is_docker:
                    payload += f"la{j + 1}:300{j + 1},"
                else:
                    payload += f"127.0.0.1:300{j + 1},"

        payloads.append(payload[:-1])  # remove trailing ','

    return payloads
