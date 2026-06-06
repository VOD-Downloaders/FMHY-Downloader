#!/bin/bash

set -e

# --- Variables ---
PUID=${PUID}
PGID=${PGID}
TZ=${TZ}

CHOWN_CONFIG=${CHOWN_CONFIG}
CHOWN_OUTPUT=${CHOWN_OUTPUT}

APP_USER=${APP_USER}
APP_BIN=${APP_BIN}

echo "Starting container with PUID=$PUID, PGID=$PGID, TZ=$TZ"

# --- Timezone ---
ln -snf /usr/share/zoneinfo/$TZ /etc/localtime
echo $TZ > /etc/timezone

# --- Create group if PGID doesn't exist ---
if ! getent group "$PGID" > /dev/null 2>&1; then
    groupadd -g "$PGID" "$APP_USER"
fi

# --- Create user if PUID doesn't exist ---
if ! getent passwd "$PUID" > /dev/null 2>&1; then
    useradd -u "$PUID" -g "$PGID" -m -s /bin/bash "$APP_USER"
else
    # User exists (e.g. root=0), just grab the name
    APP_USER=$(getent passwd "$PUID" | cut -d: -f1)
fi

# --- Change ownership of APP_BIN ---
chown ${APP_USER}:${APP_USER} /app/${APP_BIN}

# --- Change ownership of output folder ---
mkdir -p /config
mkdir -p /output

if [ $CHOWN_CONFIG ]; then
  chown -R ${APP_USER}:${APP_USER} /config
fi
if [ $CHOWN_OUTPUT ]; then
  chown -R ${APP_USER}:${APP_USER} /output
fi

# --- Start virtual X display ---
Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp &
export DISPLAY=:99

for _ in $(seq 1 40); do # Wait for the socket
    [ -S /tmp/.X11-unix/X99 ] && break
    sleep 0.25
done

if [ ! -S /tmp/.X11-unix/X99 ]; then
    echo "[ERROR]: Xvfb failed to start on :99" >&2
    exit 1
fi

# --- Drop privileges ---
exec gosu "$PUID:$PGID" "$@"
