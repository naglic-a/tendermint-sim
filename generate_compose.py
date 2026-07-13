import yaml

NUM_NODES = 50
PEERS_OUT = 3 # Each node connects to 3 other nodes(GOSSIP)

services = {}

for i in range(NUM_NODES):
    peers = []
    # Connect to the next PEERS_OUT nodes to guarantee a connected topology
    for j in range(1, PEERS_OUT + 1):
        target = (i + j) % NUM_NODES
        target_port = 8000 + target
        peers.append(f"{target}:node{target}:{target_port}")
    
    peers_str = ",".join(peers)
    if(i < 15):
        behaviour = "double-vote"
    else:
        behaviour = "standard"
    # Add the monitor service first
    if i == 0:
        services["monitor"] = {
            "build": {
                "context": ".",
                "dockerfile": "Dockerfile.monitor"
            },
            "ports": [
                "8080:8080",
                "8081:8081"
            ],
            "networks": ["tm-net"]
        }

    services[f"node{i}"] = {
        "build": ".",
        "image": "tendermint-sim-image",
        "cap_add": ["NET_ADMIN"],
        "environment": [
            f"NODE_ID={i}",
            f"TOTAL_NODES={NUM_NODES}",
            f"PEERS={peers_str}",
            "RUST_LOG=info",
            f"BEHAVIOR={behaviour}",
            "LATENCY_MS=500"
        ],
        "networks": ["tm-net"]
    }

compose = {
    "services": services,
    "networks": {
        "tm-net": {
            "driver": "bridge"
        }
    }
}

with open("docker-compose.yml", "w") as f:
    yaml.dump(compose, f, sort_keys=False)

print(f"Successfully generated docker-compose.yml for {NUM_NODES} nodes with {PEERS_OUT} peers each")
