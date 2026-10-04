# How to Install and Use Telescrap

## Telegram bot creation

1. Create a Telegram bot and get its token (see [this documentation](https://core.telegram.org/bots#6-botfather)).

2. Get your Telegram chat ID and save it for later.

## Set up your running server

To run the bot, you need a server — either a physical machine (like a Raspberry Pi) or a VPS from a hosting provider.

### Raspberry Pi setup

In this section, we will see how to structure your Raspberry Pi to run one or multiple instances of the bot.

### Docker compose

The easiest way to run multiple instances of the bot is to use Docker and Docker Compose.

To do so, follow the instructions in [DOCKER_DEPLOYMENT.md](./DOCKER_DEPLOYMENT.md) to set up the Docker environment and deploy the bot.

