## Requirements
- Arch: x86-64
- RAM: 2 GB
- Disk space: 20 GB
- Docker and docker-compose installed

## Docker
* Clone the repo
    ```bash
    git clone https://github.com/mmm29/kanban_backend
    cd kanban_backend
    ```
* Inside the repository folder run `docker-compose -f ./deployment/docker-compose.yml up`

The API server will be listening on port `35124`.