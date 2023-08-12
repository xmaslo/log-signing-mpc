IS_DOCKER = False

BASE_URL = "http://localhost"

SERVER_PORT1 = "8000"
SERVER_PORT2 = "8001"
SERVER_PORT3 = "8002"

if IS_DOCKER:
    URL0 = "la1:3000"
    URL1 = "la2:3001"
    URL2 = "la3:3002"
else:
    URL0 = "127.0.0.1:3000"
    URL1 = "127.0.0.1:3001"
    URL2 = "127.0.0.1:3002"