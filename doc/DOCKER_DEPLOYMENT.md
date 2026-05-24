
### 1. Install Docker and Docker Compose on your server.

#### Detailed setup for Raspberry Pi 3/4 (aarch64 / arm64)

Use this procedure on a Raspberry Pi running Raspberry Pi OS (64-bit).

1. Update system packages:

```bash
sudo apt update
sudo apt upgrade -y
```

2. Confirm your architecture is aarch64 (expected output: `aarch64`):

```bash
uname -m
```

If the output is `armv7l`, use Raspberry Pi OS 32-bit instructions (`armhf`) instead.

3. Install prerequisites:

```bash
sudo apt install -y ca-certificates curl gnupg
```

4. Add Docker official GPG key and repository:

```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

echo \
   "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
   $(. /etc/os-release && echo $VERSION_CODENAME) stable" | \
   sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
```

5. Install Docker Engine, CLI, container runtime, and Docker Compose plugin:

```bash
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

6. Enable Docker at boot and start it now:

```bash
sudo systemctl enable docker
sudo systemctl start docker
```

7. (Optional but recommended) Run Docker without `sudo`:

```bash
sudo usermod -aG docker $USER
```

Then log out and log in again (or reboot) to apply group changes.

Official reference: https://docs.docker.com/engine/install/debian/

### 2. (Optional) Get example file template for docker compose, environment variables and default configuration for the bot.

If you want a clean checkout with only the current template files, use Git sparse-checkout:

```bash
git clone --filter=blob:none --sparse git@github.com:ficaud/telescrap-sr.git telescrap-templates
cd telescrap-templates
git checkout main

git sparse-checkout init --no-cone
git sparse-checkout set examples/docker
git read-tree -mu HEAD

ls -la examples/docker
```

You should get the current templates from:

- `examples/docker/docker-compose.yml.example`
- `examples/docker/.env.example`
- `examples/docker/config_scan.json.example`

Then copy them into your deployment directory:

###  3. Create a docker compose file (`docker-compose.yml`) in the root of the project with the content of the template provided in the repository: `examples/docker/docker-compose.yml.example`.

```bash
mv examples/docker/docker-compose.yml.example docker-compose.yml
nano docker-compose.yml
```

Docker compose example template with minimum configuration for 1 instance of the bot is provided in the repository (`docker-compose.yml.example`):

```yaml
services:
  # Scanner Instance 1 - Port 3000
  scanner-1:
    image: ${DOCKER_IMAGE:-ghcr.io/ficaud/telescrap-sr:latest}
    container_name: telescrap-scanner-1
    ports:
      - "3000:3000"
    environment:
      - TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN_SCANNER_1}
      - TELEGRAM_CHAT_ID=${TELEGRAM_CHAT_ID_SCANNER_1}
      - ADMIN_PANEL_PORT=3000
      - SHOP_EMAIL=${SHOP_EMAIL}
      - SHOP_PASSWORD=${SHOP_PASSWORD}
      - RUST_LOG=${RUST_LOG:-info}
      - INSTANCE_ID=scanner-1
      - MATCHS_DB_PATH=/app/data/matchs.db
    env_file:
      - .env
    volumes:
      - ./data/scanner-1:/app/data
    restart: unless-stopped

volumes:
  scanner-1-data:

networks:
  default:
    name: telescrap-network
    driver: bridge
```

###  4. Copy the environment template and edit it with your Telegram credentials:

```bash
mv examples/docker/.env.example .env
nano .env
```

###  5. Fill in the `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` fields with your actual Telegram bot token and chat ID.

This is where all the configuration for the different instances of the bot will be stored.

Example of the `.env` file content with the required Telegram credentials for 1 instance of the bot:

```bash
DOCKER_IMAGE=ghcr.io/ficaud/telescrap-sr:latest
TELEGRAM_BOT_TOKEN_SCANNER_1=<your_scanner1_bot_token_here>
TELEGRAM_CHAT_ID_SCANNER_1=<your_scanner1_chat_id_here>
RUST_LOG=info
```


###  6. Copy the config_scan.json template and edit it with your desired default start configuration

This configuration will be the same for all instances, but can be updated at runtime from the admin panel.

```bash
cp examples/docker/config_scan.json.example config_scan.json
nano config_scan.json
```

Configuration json example template is provided in the repository (`examples/docker/config_scan.json.example`):

```json
{
    "mode": "Passive",
    "interval": 60,
    "club": "StadeRochelais",
    "nature": "Rugby",
    "is_preview": true,
    "filter_chain": [
        {
            "type": "Encounter",
            "name": ""
        },
        {
            "type": "Price",
            "min": 10.0,
            "max": 50.0
        },
        {
            "type": "Seat",
            "category": null,
            "bloc": null,
            "row": null,
            "min_consecutive": 2
        }
    ]
}
```

###  7. Start the Docker services:

```bash
docker compose up -d
```

###  8. Your scanner instances are now running:
   - **Scanner 1** (Telegram notifications): `http://<your_server_ip>:3000`
   - **Scanner 2** if any (Telegram notifications): `http://<your_server_ip>:3001`
   
Both instances will continuously monitor for resale tickets and send instant Telegram notifications when matches are found.

## Useful docker commands

```bash
# Down and remove volumes and orphans
docker compose down -v --remove-orphans
```

```bash
# Rebuild images without cache and restart
docker compose build --no-cache 
```

```bash
# Start volumes and services in detached mode
docker compose up -d
```

