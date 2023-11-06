#!/bin/bash
set -ex
source "$HOME/.cargo/env"

cd /root/mnt/rust-gpu



# the watches don't work on Windows + WSL2 + Docker
# https://github.com/microsoft/WSL/issues/4739
# /root/.cargo/bin/cargo  run --release -- "crates/shader" -w ./crates/shader/src -w ../bevy-app/crates/viewer/entry_points.json ../bevy-app/crates/viewer/assets/rust-gpu/shader.rust-gpu.msgpack

set +x
export TS_FILE=/tmp/latest-timestamp
while true; do
    LATEST_TS=$(find ../bevy-app/crates/viewer/entry_points.json  ./crates/shader/ -type f -exec stat \{} --printf="%y\n" \;  | sort -nr | head -n1)
    OLD_TS=$(cat $TS_FILE||echo '')
    if [ "$LATEST_TS" == "$OLD_TS" ]; then
        sleep 2
    else
         /root/.cargo/bin/cargo  run --release -- "crates/shader"  ../bevy-app/crates/viewer/assets/rust-gpu/shader.rust-gpu.msgpack
         echo "$LATEST_TS" > $TS_FILE
    fi
done