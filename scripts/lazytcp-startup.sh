#!/usr/bin/env bash

if [[ -z "$SERVER_PORT" ]]; then
    echo "SERVER_PORT is not set, default to 25565"
    SERVER_PORT=25565
fi

if [[ -z "$DEBOUNCE_MILLIS" ]]; then
    echo "DEBOUNCE_MILLIS is not set, default to 60000"
    DEBOUNCE_TIME_MILLIS=60000
fi

ORIGINAL_SERVER_PORT="$SERVER_PORT"
export SERVER_PORT=$((SERVER_PORT+1))

"$HOME/.cargo/bin/lazytcp" --command "/start" \
    --downstream-addr "127.0.0.1:$SERVER_PORT" \
    --listen-addr "0.0.0.0:$ORIGINAL_SERVER_PORT" \
    --stdout-ready-pattern "RCON running" \
    --shutdown-stdin-command "stop" \
    --debounce-time-millis "$DEBOUNCE_MILLIS"
