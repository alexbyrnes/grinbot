# script to start the node, wallet owner API,
# and the bot in docker
cd /node
grin server run &
cd /mywallet
grin-wallet -p pass owner_api &
cd /GrinBot
cargo run
