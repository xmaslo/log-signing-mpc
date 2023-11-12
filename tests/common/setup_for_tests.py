IS_DOCKER = True

BASE_URL = "http://127.0.0.1"


def get_ports(n, port):
    ports = []
    port_number = port
    for i in range(1, n + 1):
        port_number += 1
        ports.append(str(port_number))

    return ports


def create_urls(n, port, is_docker, url):
    urls = []
    port_number = port
    for i in range(1, n + 1):
        port_number += 1
        if is_docker:
            urls.append(f"la{i}:{port_number}")
        else:
            urls.append(f"{url}:{port_number}")

    return urls


def get_endpoint_urls(n, is_docker):
    return create_urls(n, 8000, is_docker, BASE_URL)


def get_inter_comm_urls(n, is_docker):
    return create_urls(n, 3000, is_docker, "127.0.0.1")


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
