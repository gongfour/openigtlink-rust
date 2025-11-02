#!/bin/bash

# 40개 메시지 타입 모두에 대한 양방향 통신 테스트

echo "=========================================="
echo "OpenIGTLink 40개 메시지 타입 전체 테스트"
echo "=========================================="
echo ""

MESSAGES=(
    "bind"
    "capability"
    "colortable"
    "command"
    "get_capability"
    "get_image"
    "get_imgmeta"
    "get_lbmeta"
    "get_point"
    "get_status"
    "get_tdata"
    "get_transform"
    "image"
    "imgmeta"
    "lbmeta"
    "ndarray"
    "point"
    "polydata"
    "position"
    "qtdata"
    "rts_capability"
    "rts_image"
    "rts_status"
    "rts_tdata"
    "rts_transform"
    "sensor"
    "status"
    "stp_image"
    "stp_ndarray"
    "stp_position"
    "stp_qtdata"
    "stp_tdata"
    "stp_transform"
    "string"
    "stt_tdata"
    "tdata"
    "trajectory"
    "transform"
    "video"
    "videometa"
)

TOTAL=${#MESSAGES[@]}
PASSED=0
FAILED=0
FAILED_LIST=()
PORT=20000

echo "테스트 시작: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

for msg_type in "${MESSAGES[@]}"; do
    PORT=$((PORT + 1))
    printf "[%2d/%2d] %-20s ... " $((PASSED + FAILED + 1)) $TOTAL "$msg_type"
    
    # 서버 시작
    cargo run --quiet --bin openigtlink -- server \
        --listen 127.0.0.1:$PORT \
        --send-enable \
        --send-message-file "examples/messages/${msg_type}.json" \
        --send-repeat-count 1 \
        --log-level error 2>/dev/null &
    
    SERVER_PID=$!
    sleep 0.8
    
    # 클라이언트 시작 및 수신
    if timeout 5 cargo run --quiet --bin openigtlink -- client \
        --connect 127.0.0.1:$PORT \
        --receive-enable \
        --receive-max-count 1 \
        --receive-timeout-sec 3 \
        --log-level error 2>/dev/null > /dev/null; then
        echo "✓"
        ((PASSED++))
    else
        echo "✗"
        ((FAILED++))
        FAILED_LIST+=("$msg_type")
    fi
    
    # 프로세스 정리
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    sleep 0.3
done

# 결과 출력
echo ""
echo "=========================================="
echo "테스트 완료: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo "전체 테스트:  $TOTAL개"
echo "통과:        $PASSED개 ✓"
echo "실패:        $FAILED개 ✗"
echo ""

if [ $FAILED -gt 0 ]; then
    echo "실패한 메시지 타입:"
    for msg in "${FAILED_LIST[@]}"; do
        echo "  - $msg"
    done
fi

echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo "✓ 모든 테스트 통과!"
    exit 0
else
    echo "✗ 일부 테스트 실패"
    exit 1
fi
