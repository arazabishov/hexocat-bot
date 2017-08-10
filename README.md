# hexocat-bot

[![Build Status](https://travis-ci.org/ArazAbishov/hexocat-bot.svg?branch=master )](https://github.com/ArazAbishov/hexocat-bot) 

Simple slash command implementation for searching GitHub repositories in Rust. This repo is a part of the blog post series:  
 - [part one](https://abishov.com/2017/07/27/hexocat-bot-part-1.html) 
 - [part two](https://abishov.com/2017/08/08/hexocat-bot-part-2.html)

In order to start the development server, use the next command:

```bash
# This command will start the Rocket server 
# that will be listening for the post requests 
# on the locahost - 0.0.0.0, port - 2727.  
cargo run
```

Here is an example of the query using cURL: 

```bash
curl -X POST \
  http://0.0.0.0:2727/hexocat/ \
  -H 'content-type: application/x-www-form-urlencoded' \
  -d 'text=retrofit&token=test_token'
```

If you want to run a server in the production environment you have either to change `Rocket.toml` file to include extra properties or to expose them as environment variables:

```bash
# Prepare release version of the binary.
cargo build --release

# Export required environment variables. 
export ROCKET_ENV=production cargo run
export ROCKET_KEY=your_slack_token
export ROCKET_PORT=2727

# Execute the binary to start server. 
target/release/hexocat-bot
```  
