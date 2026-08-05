#!/bin/sh
# Custom entrypoint: start crond in background, then exec the main process.
# busybox crond needs crontab files in a dedicated directory, one file per user.
CROND_DIR=/tmp/crond
mkdir -p "$CROND_DIR"
cp /app/crontab "$CROND_DIR/root"
crond -c "$CROND_DIR" -l 8
exec tini -- "$@"
