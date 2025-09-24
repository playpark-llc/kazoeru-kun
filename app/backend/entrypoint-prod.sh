#!/bin/sh
# Production entrypoint script

# Signal handling for graceful shutdown
trap 'kill -TERM $PID' TERM INT

# Start application
/app/backend &
PID=$!
wait $PID