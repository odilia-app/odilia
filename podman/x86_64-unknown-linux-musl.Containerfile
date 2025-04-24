FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl:latest

COPY podman/test-wrapper.sh /entrypoint.sh
COPY podman/install-deps.sh /deps.sh
RUN chmod +x /entrypoint.sh /deps.sh
RUN /deps.sh
ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
