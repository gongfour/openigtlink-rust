#!/bin/bash

# 양방향 상세 테스트 - 선택된 메시지 타입 테스트

echo "=========================================="
echo "양방향 양쪽 송수신 상세 테스트"
echo "=========================================="
echo ""

# 테스트할 메시지 타입 (각 카테고리별)
TEST_CASES=(
    "transform"      # 기본 메시지
    "get_transform"  # 조회 메시지
    "rts_transform"  # 응답 메시지
    "image"          # 복잡 메시지
    "qtdata"         # 배열 메시지
)

PORT=25000

for msg_type in "${TEST_CASES[@]}"; do
    PORT=$((PORT + 1))
    
    echo "════════════════════════════════════════"
    echo "테스트: $msg_type (포트: $PORT)"
    echo "════════════════════════════════════════"
    echo ""
    
    # 테스트 디렉토리 생성 (현재 디렉토리 사용)
    TEST_DIR=".test_bidirectional_${msg_type}_$$"
    mkdir -p "$TEST_DIR"
    
    echo "[1/3] 서버 시작 (메시지 송신 + 수신 대기)..."
    cargo run --quiet --bin openigtlink -- server \
        --listen 127.0.0.1:$PORT \
        --send-enable \
        --send-message-file "examples/messages/${msg_type}.json" \
        --send-repeat-count 1 \
        --receive-enable \
        --receive-max-count 1 \
        --receive-timeout-sec 5 \
        --receive-output-file "$TEST_DIR/server_received.json" \
        --log-level warn 2>&1 | grep -E "(listening|Loaded|Received|sent)" &
    
    SERVER_PID=$!
    sleep 1.5
    
    echo "[2/3] 클라이언트 시작 (메시지 송신 + 수신)..."
    timeout 8 cargo run --quiet --bin openigtlink -- client \
        --connect 127.0.0.1:$PORT \
        --send-enable \
        --send-message-file "examples/messages/${msg_type}.json" \
        --send-repeat-count 1 \
        --receive-enable \
        --receive-max-count 1 \
        --receive-timeout-sec 5 \
        --receive-output-file "$TEST_DIR/client_received.json" \
        --log-level warn 2>&1 | grep -E "(connected|Loaded|Received|sent)"
    
    sleep 1
    
    # 프로세스 종료
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    
    echo ""
    echo "[3/3] 결과 확인..."
    echo ""
    
    # 결과 분석
    if [ -f "$TEST_DIR/server_received.json" ]; then
        echo "✓ 서버 수신: 클라이언트에서 보낸 메시지 수신 성공"
        echo "  크기: $(wc -c < $TEST_DIR/server_received.json) bytes"
        echo ""
    else
        echo "✗ 서버 수신: 실패"
        echo ""
    fi
    
    if [ -f "$TEST_DIR/client_received.json" ]; then
        echo "✓ 클라이언트 수신: 서버에서 보낸 메시지 수신 성공"
        echo "  크기: $(wc -c < $TEST_DIR/client_received.json) bytes"
        echo ""
    else
        echo "✗ 클라이언트 수신: 실패"
        echo ""
    fi
    
    # 메시지 내용 확인
    if [ -f "$TEST_DIR/server_received.json" ] && [ -f "$TEST_DIR/client_received.json" ]; then
        echo "📄 메시지 내용 확인:"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "클라이언트에서 받은 메시지 (서버가 보낸 것):"
        head -3 "$TEST_DIR/client_received.json" | sed 's/^/  /'
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
    fi
    
    # 정리
    rm -rf "$TEST_DIR"
    
    echo ""
done

echo "=========================================="
echo "✓ 양방향 상세 테스트 완료"
echo "=========================================="
