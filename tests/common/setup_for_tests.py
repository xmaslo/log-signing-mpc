IS_DOCKER = True

BASE_URL = "127.0.0.1"
BASE_DOCKER_URL = "la"
BASE_URL_HTTP = f"http://{BASE_URL}"
BASE_DOCKER_URL_HTTP = f"http://{BASE_DOCKER_URL}"


def get_ports(n, port):
    ports = []
    port_number = port
    for i in range(1, n + 1):
        port_number += 1
        ports.append(str(port_number))

    return ports


def get_endpoint_urls(n):
    urls = []
    port_number = 8000
    for i in range(1, n + 1):
        port_number += 1
        urls.append(f"{BASE_URL_HTTP}:{port_number}")
    return urls


def get_inter_comm_urls(n, is_docker):
    urls = []
    port_number = 3000
    for i in range(1, n + 1):
        port_number += 1
        if is_docker:
            urls.append(f"{BASE_DOCKER_URL}{i}:{port_number}")
        else:
            urls.append(f"{BASE_URL}:{port_number}")
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
