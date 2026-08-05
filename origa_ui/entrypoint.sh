#!/bin/sh
# Custom entrypoint: start crond in background, then exec the main process (trail).
# tini runs as PID 1 (see Dockerfile ENTRYPOINT) and handles signal forwarding + zombie reaping.
CROND_DIR=/tmp/crond
mkdir -p "$CROND_DIR"
cp /app/crontab "$CROND_DIR/root"
crond -c "$CROND_DIR" -l 8
exec "$@"
