


## Setup

```bash
spacetime init --lang=rust
spacetime server add http://localhost:7556 chatdb
spacetime publish -s http://localhost:7556 chatdb
spacetime call -s http://localhost:7556 chatdb add "wildonion"
sudo mkdir -p ../panel/spacetimedb/client/chatdb
spacetime generate --lang rust --out-dir ../panel/spacetimedb/client/chatdb --project-path .
spacetime sql -s http://localhost:7556 chatdb "SELECT * FROM Person"
spacetime logs chatdb -s http://localhost:7556
```