#!/bin/bash

# Kill any existing processes
killall -9 openigtlink 2>/dev/null || true
sleep 1

# Run server with SEND enabled
echo "Starting server with SEND enabled..."
cargo run --bin openigtlink -- server \
  --listen 127.0.0.1:19945 \
  --send-enable \
  --send-message-file examples/messages/transform.json \
  --send-repeat-count 2 \
  --send-interval-ms 500 \
  --log-level warn \
  > /tmp/server.log 2>&1 &
SERVER_PID=$!

sleep 2

# Run client
echo "Starting client..."
timeout 5 cargo run --bin openigtlink -- client \
  --connect 127.0.0.1:19945 \
  --log-level warn 2>&1 | tee /tmp/client.log || true

sleep 1

# Cleanup
kill $SERVER_PID 2>/dev/null || true
sleep 1

echo ""
echo "=== Server Output ==="
cat /tmp/server.log

echo ""
echo "=== Client Output ==="
cat /tmp/client.log

echo ""
echo "✓ SEND functionality test completed"
