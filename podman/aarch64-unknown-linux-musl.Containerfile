FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:latest

RUN apt-get update && apt-get install -y libevdev-dev linux-headers-generic clang dbus-x11 dunst xvfb
COPY ./podman/test-wrapper.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
