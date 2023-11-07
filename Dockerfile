FROM debian

RUN apt-get update -y && apt-get install -y build-essential wget curl  pkg-config  g++ pkg-config libx11-dev libasound2-dev \
   libudev-dev libwayland-dev libxkbcommon-dev mesa-vulkan-drivers  wget curl  pkg-config \
   && wget https://sh.rustup.rs -O /tmp/rustup.sh && chmod +x /tmp/rustup.sh \
   && sh /tmp/rustup.sh -y --default-toolchain none --no-update-default-toolchain

RUN /root/.cargo/bin/rustup install nightly-2023-05-27
RUN /root/.cargo/bin/rustup override set nightly-2023-05-27-x86_64-unknown-linux-gnu
RUN /root/.cargo/bin/rustup component add  rust-src rustc-dev llvm-tools-preview

RUN mkdir /docker-cargo-target
ENV CARGO_TARGET_DIR=/docker-cargo-target
ENV PATH="$PATH:/root/.cargo/bin"
WORKDIR /init-build
ADD . .
RUN bash -c "cd rust-gpu && cargo run --release -- crates/shader ../bevy-app/crates/viewer/assets/rust-gpu/shader.rust-gpu.msgpack"

WORKDIR /root/mnt/