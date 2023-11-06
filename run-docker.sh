#!/bin/bash
set -ex

# fix docker when using Docker + Git for Windows Bash (mingw)
export MSYS_NO_PATHCONV=1

export IMGNAME="rust-gpu-builder:local"
export CONTAINER="example-rust-gpu-builder"

if [[ "$(docker images -q $IMGNAME 2> /dev/null)" == "" ]]; then
    docker build . --tag $IMGNAME 
fi
( docker rm -f $CONTAINER || true;  docker run --rm --name $CONTAINER -v "$(pwd):/root/mnt" $IMGNAME /root/mnt/rust-gpu/entrypoint.sh  )
