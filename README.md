# GrinBot

GrinBot is a self-hosted Telegram bot wallet for the [Grin](https://grin-tech.org/) cryptocurrency. You can run GrinBot on your own hardware, and interact with it through Telegram messages.

# Getting started

## Building

### Docker

```shell
git clone --branch v0.0.1 https://github.com/alexbyrnes/GrinBot.git
cd GrinBot
docker build -t grinbot .
docker run -it grinbot
```

### From source

```shell
git clone --branch v0.0.1 https://github.com/alexbyrnes/GrinBot.git
cargo install --path GrinBot --locked
```

### From source &mdash; no install
```shell
git clone --branch v0.0.1 https://github.com/alexbyrnes/GrinBot.git
cd GrinBot
cargo run
```

### Testing

```shell
cargo test
```

### Rust docs
```shell
cargo doc --no-deps --document-private-items --open
```
## Running and interacting without a Telegram account

All commands can be executed locally without a Telegram account.

```shell
grinbot -c "/help"
```

## Running and interacting with your Telegram account

### Requirements

* A Telegram account with username. [Download](https://telegram.org/)
* A bot instance. [Instructions](https://core.telegram.org/bots#6-botfather)
* The Grin Wallet Owner API. [Repository](https://github.com/mimblewimble/grin-wallet)

Once your bot instance is [created](https://core.telegram.org/bots#6-botfather) you should receive a message with your token. Enter the token and your username in [config.yml](config.yml).

Start the bot by running `grinbot` in a directory with config.yml and logging.yml, or `cargo run` in the root of the repository. Go to the link provided by Telegram on the device or desktop where Telegram is installed. (The link starts with `https://t.me/`.) You should get a prompt to open a chat with your bot.

Type and send `/help` for a list of commands.

Note: The best source of troubleshooting information is the [dockerfile](dockerfile) where a complete bot with Grin node and wallet is set up from scratch.

## Architecture and Security

GrinBot uses the Telegram bot long polling interface. This means there's no need for an externally-accessible IP or port. GrinBot will connect to Telegram and pull new messages (called [Updates](https://core.telegram.org/bots/api#getting-updates)) from an endpoint specifically for your bot instance using your token. To get an idea of how this works, you can visit `https://api.telegram.org/bot<your api token>/getUpdates` to manually consume messages you have sent your bot. This is the address GrinBot polls.

The only information that is sent to Telegram is the contents of the chat itself &mdash; the commands you send to your bot and the messages it sends back. The commands and replies do not include passwords or tokens.

Telegram bot traffic is _not_ end-to-end encrypted, however Telegram claims [GDPR compliance](https://telegram.org/faq#q-what-about-gdpr) and the ability to [delete messages](https://telegram.org/faq#q-can-i-delete-my-messages). If you are using GrinBot for purposes that require stronger security guarantees than these, you should not use this version.


## Roadmap

* Local command interface
* Command aliases
* Confirmation dialog
* Usage help screens
* `grinbot init` for default config files

## Contributing

Contributions are welcome. Please submit an issue, or claim an existing one for visibility, and PR against the develop branch.

