# Images for installer/iso/build. Two targets:
#   rust: builds the dotinstall binaries for aarch64 and x86_64 Linux
#   iso:  builds the live ISO (mmdebstrap + casper + grub + xorriso)

FROM rust:1-trixie AS rust
RUN apt-get update && apt-get install -y --no-install-recommends \
      gcc-x86-64-linux-gnu libc6-dev-amd64-cross \
      gcc-aarch64-linux-gnu libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

FROM ubuntu:26.04 AS iso
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      mmdebstrap arch-test ca-certificates squashfs-tools xorriso mtools dosfstools zstd \
      && rm -rf /var/lib/apt/lists/*
