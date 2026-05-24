
## 1. Create directories

The binary is shared between all instances, while each instance has its own directory for its configuration:

```bash
mkdir ~/telescrap
mkdir ~/telescrap/instances
mkdir ~/telescrap/instances/bot1
```

## 2. Copy the binary

```bash
scp target/aarch64-unknown-linux-gnu/release/telescrap-sr user@your-server-ip:/home/user/telescrap/
```

## 3. Create the .env file

Create a `.env` file in the `bot1` directory with the following content:

```env
TELEGRAM_BOT_TOKEN=your-telegram-bot-token
TELEGRAM_CHAT_ID=your-telegram-chat-id

# Required only for aggressive scan mode
SHOP_EMAIL=your-shop-email
SHOP_PASSWORD=your-shop-password

# Admin panel port for this instance (default: 3000)
ADMIN_PANEL_PORT=3000
```

## 4. Create the systemd service

Create a service file at `/etc/systemd/system/telescrap-bot1.service` with the following content:

```ini
[Unit]
Description=Telescrap Bot 1
After=network.target

[Service]
WorkingDirectory=/home/user/telescrap/instances/bot1
ExecStart=/home/user/telescrap/telescrap-sr
EnvironmentFile=/home/user/telescrap/instances/bot1/.env
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Here is another example if you're running a second instance (`bot2`):

```ini
[Unit]
Description=Telescrap bot instance %i
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=jfi
Group=jfi
WorkingDirectory=/home/user/telescrap/instances/%i
EnvironmentFile=/home/user/telescrap/instances/%i/.env
ExecStart=/home/user/telescrap/telescrap-sr
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
````

**5. Start the service**

Reload systemd and enable the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable telescrap-bot1
sudo systemctl start telescrap-bot1
```

## Useful management commands

```bash
# Check the status of the service
sudo systemctl status telescrap-bot1

# View live logs
sudo journalctl -u telescrap-bot1 -f

# Restart the service (e.g. after updating the binary)
sudo systemctl restart telescrap-bot1

# Stop the service
sudo systemctl stop telescrap-bot1

# Disable autostart on boot
sudo systemctl disable telescrap-bot1
```
