# telescrap-sr
<br>
<p align="center">
	<img src="doc/img/logo.png" width="150">
</p>

[![Cross-Compile and Deploy](https://github.com/Thejulfi/telescrap-sr/actions/workflows/deploy.yml/badge.svg)](https://github.com/Thejulfi/telescrap-sr/actions/workflows/deploy.yml)

Scraping tool to get notification for resale ticket, currently implemented for Stade Rochelais rugby matches.

## Roadmap

- [ ] Filter rework to allow 1 passive and mulitple aggressive filters
- [x] Admin panel to manage the bot (web interface or terminal)
- [x] config.json to set up the scan and filter configuration by default (and save updated ones from the admin panel).
- [ ] Web app to collect match ticket requests and build a waiting list that the bot can use to automatically add tickets to cart in aggressive mode.

## Why this project ?

Rugby club subscriptions are saturated, making it nearly impossible for new fans to get tickets. While resale platforms exist, they're overwhelming and tedious to monitor.

This bot automates the process by continuously scanning for available resale tickets and instantly notifying you via Telegram, so you never miss an opportunity to see your favorite team play.

## How does it work?

The bot analyzes the homepage of the ticketing site, looking for the matches that are currently resaling tickets. When it finds a match, it checks if the tickets are available for resale and if they are, it sends a notification to the Telegram channel with the details of the match and the price of the tickets.

## Configuration

To set up the bot, follow the steps describes in [SERVER_INSTALLATION.md](doc/SERVER_INSTALLATION.md) documentation.

## 🚨 Telegram Resale Channel 🚨

The bot is currently active on a private resale channel, accessible only by manual addition.
You can request access by contacting the project administrator via private message.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="doc/img/telegram_channel.png">
  <source media="(prefers-color-scheme: light)" srcset="doc/img/telegram_channel.png">
  <img src="doc/img/telegram_channel.png" width="100%">
</picture>

## Web Admin Panel

The bot includes a web interface for managing its operations, allowing you to start or stop the bot, configure filters and other settings without needing to modify the configuration file.

Read the [ADMIN_PANEL.md](doc/ADMIN_PANEL.md) documentation for more details about the features of the admin panel and how to use it.

## See also

- [ARCHITECTURE.md](doc/ARCHITECTURE.md) : for more details about the architecture of the project and the crates organization.
- [CHANGELOG.md](CHANGELOG.md) : for a detailed list of changes and updates made to the project over time.