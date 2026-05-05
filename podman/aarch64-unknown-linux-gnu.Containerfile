FROM --platform=linux/arm64 arm64v8/ubuntu:24.04

COPY podman/test-wrapper.sh /entrypoint.sh
COPY podman/install-deps.sh /deps.sh
RUN chmod +x /entrypoint.sh /deps.sh
RUN /deps.sh
ENTRYPOINT ["dbus-launch", "/entrypoint.sh"]
