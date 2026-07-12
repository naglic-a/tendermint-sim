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
    
    services[f"node{i}"] = {
        "build": ".",
        "environment": [
            f"NODE_ID={i}",
            f"TOTAL_NODES={NUM_NODES}",
            f"PEERS={peers_str}",
            "RUST_LOG=info", # Changed from debug to info to keep logs readable
            "BEHAVIOR=standard"
        ],
        "networks": ["tm-net"]
    }

compose = {
    "version": "3.8",
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
