# How to Install and Use Telescrap

## Telegram bot creation

1. Create a Telegram bot and get its token (see [this documentation](https://core.telegram.org/bots#6-botfather)).

2. Get your Telegram chat ID and save it for later (see [this documentation](https://stackoverflow.com/questions/32423837/telegram-bot-how-to-get-a-group-chat-id)).


## Compilation and deployment (manual)

**SKIP THIS STEP IF YOU ARE USING THE RELEASE BUILDS FROM THE GITHUB RELEASES PAGE, AS THEY ARE ALREADY COMPILED FOR ARM64.**

*Pre-requisite: Rust and Cargo must be installed on your machine.*

Install the `cargo-zigbuild` tool and the stable Rust toolchain to enable cross-compilation:

```bash
cargo install cargo-zigbuild
rustup install stable
```

To deploy on a Raspberry Pi 3/4, target the `aarch64-unknown-linux-gnu` architecture:

```bash
rustup target add aarch64-unknown-linux-gnu
cargo zigbuild --release --target aarch64-unknown-linux-gnu
```

The compiled binary will be at `target/aarch64-unknown-linux-gnu/release/telescrap-sr`. Copy it to your server:

```bash
scp target/aarch64-unknown-linux-gnu/release/telescrap-sr user@your-server-ip:/home/user/telescrap/
```

## Set up your running server

To run the bot, you need a server — either a physical machine (like a Raspberry Pi) or a VPS from a hosting provider.

### Raspberry Pi setup

In this section, we will see how to structure your Raspberry Pi to run one or multiple instances of the bot.

### 1. Docker compose

The easiest way to run multiple instances of the bot is to use Docker and Docker Compose.

To do so, follow the instructions in [DOCKER_DEPLOYMENT.md](./DOCKER_DEPLOYMENT.md) to set up the Docker environment and deploy the bot.

### 2. Manual setup

If you prefer to run the bot without Docker, you can follow these steps [MANUAL_DEPLOYMENT.md](./MANUAL_DEPLOYMENT.md) to set up the bot manually on your server.