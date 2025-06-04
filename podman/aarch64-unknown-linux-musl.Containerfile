FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:edge

COPY podman/test-wrapper.sh /entrypoint.sh
COPY podman/install-deps.sh /deps.sh
RUN chmod +x /entrypoint.sh /deps.sh
RUN /deps.sh
ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
