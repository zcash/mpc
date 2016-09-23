#!/bin/sh

set -eu

cd iso/musl-builder
sudo docker build -t mpc-musl-builder .
cd ../../

sudo docker run --rm -it -v "$(pwd)":/home/rust/src mpc-musl-builder cargo build --release --bin compute --no-default-features
cp target/x86_64-unknown-linux-musl/release/compute iso/mpc_compute/mpc_compute.rs

cd iso
sudo docker build -t mpc-iso .
cd ..

sudo docker run --rm -it -v "$(pwd)":/home/builder/target mpc-iso cp -L alpine-compute.iso /home/builder/target/alpine-compute.iso
