#!/bin/bash
sudo chown -R root:root . && sudo chmod -R 777 .
cargo build --bin conse --release
sudo rm conse
sudo cp target/release/conse ./conse
sudo pm2 delete conse
sudo pm2 start conse --name conse