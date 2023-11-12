IS_DOCKER = True

BASE_URL = "http://127.0.0.1"

SERVER_PORT1 = "8001"
SERVER_PORT2 = "8002"
SERVER_PORT3 = "8003"
SERVER_PORT4 = "8004"
SERVER_PORT5 = "8005"
SERVER_PORT6 = "8006"

if IS_DOCKER:
    URL1 = "la1:3001"
    URL2 = "la2:3002"
    URL3 = "la3:3003"
    URL4 = "la4:3004"
    URL5 = "la5:3005"
    URL6 = "la6:3006"
else:
    URL1 = "127.0.0.1:3001"
    URL2 = "127.0.0.1:3002"
    URL3 = "127.0.0.1:3003"
    URL4 = "127.0.0.1:3004"
    URL5 = "127.0.0.1:3005"
    URL6 = "127.0.0.1:3006"


def get_urls(n):
    urls = []
    for i in range(1, n + 1):
        urls.append(f"{BASE_URL}:800{i}")

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
