#!/bin/bash

echo "========================================"
echo "간단한 양방향 통신 데모"
echo "========================================"
echo ""

# 테스트 1: TRANSFORM 메시지 양방향
echo "📝 테스트 1: TRANSFORM 메시지 양방향 통신"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Flow: [Server 송신 TRANSFORM] → [Client 수신]"
echo "      [Client 송신 TRANSFORM] → [Server 수신]"
echo ""

echo "[1] 서버 시작..."
cargo run --quiet --bin openigtlink -- server \
    --listen 127.0.0.1:26001 \
    --send-enable \
    --send-message-file examples/messages/transform.json \
    --send-repeat-count 1 \
    --receive-enable \
    --receive-max-count 1 \
    --receive-timeout-sec 5 \
    --log-level info 2>&1 | grep -E "(listening|Loaded|Received|Message)" &

SERVER_PID=$!
sleep 2

echo "[2] 클라이언트 시작..."
echo ""
timeout 8 cargo run --quiet --bin openigtlink -- client \
    --connect 127.0.0.1:26001 \
    --send-enable \
    --send-message-file examples/messages/transform.json \
    --send-repeat-count 1 \
    --receive-enable \
    --receive-max-count 1 \
    --receive-timeout-sec 5 \
    --log-level info 2>&1 | grep -E "(connected|Loaded|Received|Message)"

sleep 2

echo ""
echo "[3] 정리..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "✓ 테스트 1 완료"
echo ""
echo ""

# 테스트 2: 서버는 STATUS, 클라이언트는 POSITION 보냄
echo "📝 테스트 2: 다른 메시지 타입 양방향 통신"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Flow: [Server 송신 STATUS] → [Client 수신]"
echo "      [Client 송신 POSITION] → [Server 수신]"
echo ""

echo "[1] 서버 시작 (STATUS 송신)..."
cargo run --quiet --bin openigtlink -- server \
    --listen 127.0.0.1:26002 \
    --send-enable \
    --send-message-file examples/messages/status.json \
    --send-repeat-count 1 \
    --receive-enable \
    --receive-max-count 1 \
    --receive-timeout-sec 5 \
    --log-level info 2>&1 | grep -E "(listening|Loaded|Received|Message)" &

SERVER_PID=$!
sleep 2

echo "[2] 클라이언트 시작 (POSITION 송신)..."
echo ""
timeout 8 cargo run --quiet --bin openigtlink -- client \
    --connect 127.0.0.1:26002 \
    --send-enable \
    --send-message-file examples/messages/position.json \
    --send-repeat-count 1 \
    --receive-enable \
    --receive-max-count 1 \
    --receive-timeout-sec 5 \
    --log-level info 2>&1 | grep -E "(connected|Loaded|Received|Message)"

sleep 2

echo ""
echo "[3] 정리..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "✓ 테스트 2 완료"
echo ""
echo "========================================"
echo "✓ 양방향 통신 데모 완료"
echo "========================================"
