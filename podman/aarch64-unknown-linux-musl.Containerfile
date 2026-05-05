FROM arm64v8/ubuntu:24.04

COPY podman/test-wrapper.sh /entrypoint.sh
COPY podman/install-deps.sh /deps.sh
RUN chmod +x /entrypoint.sh /deps.sh
RUN /deps.sh
RUN DEBIAN_FRONTEND=noninteractive apt install -y musl-tools musl-dev

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    CC_aarch64_unknown_linux_musl=musl-gcc

ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
