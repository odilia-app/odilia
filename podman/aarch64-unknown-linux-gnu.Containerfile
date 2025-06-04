FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:edge

COPY podman/install-deps.sh /deps.sh
RUN chmod +x /deps.sh
RUN /deps.sh
COPY podman/test-wrapper.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
