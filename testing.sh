CROSS_CONTAINER_ENGINE="podman"  CROSS_CONTAINER_ENGINE_NO_BUILDKIT=1 CROSS_CONTAINER_OPTS="--group-add keep-groups -v /dev/input:/dev/input:z,dev -v /dev/uinput:/dev/uinput:z,dev --tty" cross --verbose test --features integration_tests -p odilia-input-server-keyboard --target x86_64-unknown-linux-musl -- --nocapture
# CROSS_CONTAINER_UID="1001" CROSS_CONTAINER_GID="993"
