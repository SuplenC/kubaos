# The "live system" for engine-e2e: the tools the engine runs on the ISO.
FROM ubuntu:26.04
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gdisk fdisk parted util-linux cryptsetup-bin btrfs-progs dosfstools e2fsprogs \
      squashfs-tools zstd tar jq ca-certificates \
    && rm -rf /var/lib/apt/lists/*
