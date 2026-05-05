FROM --platform=linux/arm64 arm64v8/ubuntu:24.04

COPY podman/test-wrapper.sh /entrypoint.sh
COPY podman/install-deps.sh /deps.sh
RUN chmod +x /entrypoint.sh /deps.sh
RUN /deps.sh
RUN DEBIAN_FRONTEND=noninteractive apt install -y musl-tools musl-dev \
    autoconf automake libtool pkg-config make
RUN ln -s /usr/include/linux        /usr/include/aarch64-linux-musl/linux \
 && ln -s /usr/include/asm-generic  /usr/include/aarch64-linux-musl/asm-generic \
 && ln -s /usr/include/aarch64-linux-gnu/asm /usr/include/aarch64-linux-musl/asm

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    CC_aarch64_unknown_linux_musl=musl-gcc

ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
