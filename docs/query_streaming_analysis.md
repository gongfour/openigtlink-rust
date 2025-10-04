# 쿼리 메시지와 스트리밍 제어의 필요성 분석

## 📋 개요

OpenIGTLink Protocol v3.0에서 정의된 쿼리 및 스트리밍 제어 메시지의 필요성과 구현 우선순위를 분석합니다.

## 🔍 메시지 타입 설명

### 1. GET_* (쿼리 메시지)

**목적**: 단일 데이터를 요청

**동작 방식**:
- 클라이언트 → 서버: `GET_<datatype>` 전송
- 서버 → 클라이언트: `<datatype>` 메시지로 응답
- 데이터 없을 시: body size = 0인 메시지 반환

**예시**:
```
Client: GET_TRANSFORM (device: "Tracker")
Server: TRANSFORM (device: "Tracker", matrix: [...])
```

### 2. STT_* / STP_* (스트리밍 제어)

**목적**: 연속 데이터 스트림 시작/중지

**동작 방식**:
- 클라이언트 → 서버: `STT_<datatype>` (스트림 시작)
- 서버 → 클라이언트: `RTS_<datatype>` (수신 확인)
- 서버 → 클라이언트: 연속적인 `<datatype>` 메시지
- 클라이언트 → 서버: `STP_<datatype>` (스트림 중지)
- 서버 → 클라이언트: `RTS_<datatype>` (중지 확인)

**예시**:
```
Client: STT_TDATA (device: "Tracker")
Server: RTS_TDATA (status: OK)
Server: TDATA, TDATA, TDATA... (60Hz)
Client: STP_TDATA (device: "Tracker")
Server: RTS_TDATA (status: OK)
```

### 3. RTS_* (Ready To Send)

**목적**: 서버의 쿼리 수신 확인

**사용 시나리오**:
- GET_ 쿼리 응답 확인
- STT_/STP_ 제어 수신 확인

## 📊 프로토콜별 지원 현황

| 메시지 타입 | GET | STT/STP/RTS | 설명 |
|------------|-----|-------------|------|
| IMAGE | ✅ | ✅ | 영상 스트리밍 |
| TRANSFORM | ✅ | ✅ | 변환 행렬 |
| POSITION | ✅ | ✅ | 위치/방향 |
| TDATA | ✅ | ✅ | 추적 데이터 |
| QTDATA | ❌ | ✅ | 쿼터니언 추적 |
| NDARRAY | ✅ | ✅ | N차원 배열 |
| IMGMETA | ✅ | ❌ | 영상 메타데이터 |
| LBMETA | ✅ | ❌ | 라벨 메타데이터 |
| POINT | ✅ | ❌ | 포인트 데이터 |
| TRAJ | ✅ | ❌ | 궤적 데이터 |
| CAPABILITY | ✅ | ❌ | 능력 조회 |
| STATUS | ✅ | ❌ | 상태 조회 |
| COMMAND | ❌ | RTS만 | 명령 응답 |
| POLYDATA | ✅ | ✅ | 폴리곤 데이터 |

## 🎯 필요성 분석

### ✅ **필수적인 경우**

1. **Pull 기반 아키텍처**
   - 클라이언트가 필요할 때만 데이터 요청
   - 서버는 수동적으로 대기
   - 예: `GET_STATUS`, `GET_CAPABILITY`

2. **대역폭 제어**
   - 스트리밍 on/off 제어 필요
   - 고주파 데이터 (60Hz+ 추적)
   - 예: `STT_TDATA` → 추적 시작, `STP_TDATA` → 중지

3. **다중 클라이언트 환경**
   - 각 클라이언트가 독립적으로 스트림 제어
   - 리소스 최적화

### ⚠️ **선택적인 경우**

1. **Push 기반으로 대체 가능**
   ```rust
   // GET_* 대신
   client.send(TransformMessage::default())?;

   // STT_/STP_ 대신
   loop {
       if should_stream {
           client.send(tracking_data)?;
       }
   }
   ```

2. **애플리케이션 레벨 제어**
   - COMMAND 메시지로 제어 가능
   - STRING 메시지로 명령 전송

3. **현대적 패턴**
   - WebSocket: 자체 스트림 제어
   - gRPC: bidirectional streaming
   - MQTT: pub/sub 패턴

## 🔄 현재 Rust 구현체의 대안

### GET_* 대안

```rust
// 방법 1: 직접 요청
client.request_transform("Tracker")?;

// 방법 2: COMMAND 메시지 사용
client.send(CommandMessage::new("GET", "TRANSFORM"))?;

// 방법 3: 애플리케이션 프로토콜
client.send(StringMessage::new("request:transform:tracker"))?;
```

### STT_/STP_ 대안

```rust
// 방법 1: COMMAND 메시지
client.send(CommandMessage::new("START_TRACKING", ""))?;
client.send(CommandMessage::new("STOP_TRACKING", ""))?;

// 방법 2: 플래그 제어
let mut streaming = true;
loop {
    if streaming {
        server.send(tracking_data)?;
    }
}

// 방법 3: Tokio channel
let (tx, rx) = mpsc::channel(100);
// 스트림 제어를 channel로
```

## 💡 권장 사항

### 우선순위 1: 필수 쿼리 메시지

**구현 필요**:
- `GET_CAPABILITY` - 프로토콜 협상에 필수
- `GET_STATUS` - 시스템 상태 확인

**이유**: 표준 프로토콜 호환성

### 우선순위 2: 선택적 쿼리

**구현 고려**:
- `GET_TRANSFORM`, `GET_IMAGE` 등
- 레거시 C++ 클라이언트 호환성 필요 시

**대안**: COMMAND 메시지로 구현

### 우선순위 3: 스트리밍 제어

**구현 불필요** (현재):
- 현대적 비동기 I/O로 충분
- Tokio의 channel/stream으로 대체
- 애플리케이션 레벨 제어 가능

**구현 필요** (향후):
- C++ OpenIGTLink 서버와 통신 필요 시
- 표준 준수가 중요한 의료기기 환경

## 📈 구현 시나리오별 필요성

| 시나리오 | GET_ | STT_/STP_ | 비고 |
|---------|------|-----------|------|
| **Rust ↔ Rust** | ❌ | ❌ | 불필요 |
| **Rust ↔ C++** | ✅ | ✅ | 호환성 필요 |
| **단일 클라이언트** | ❌ | ❌ | 직접 제어 가능 |
| **다중 클라이언트** | ⚠️ | ✅ | 제어 필요 |
| **의료기기 인증** | ✅ | ✅ | 표준 준수 |
| **프로토타입** | ❌ | ❌ | 불필요 |

## 🎬 결론

### 현재 상태
Rust 구현체는 **핵심 데이터 메시지에 집중**하여 쿼리/스트리밍 제어 없이도 충분히 동작 가능

### 구현이 필요한 경우
1. **C++ OpenIGTLink와 상호운용성** 필요
2. **의료기기 인증**을 위한 표준 준수
3. **다중 클라이언트** 환경에서 리소스 관리

### 권장 접근
- **Phase 1**: GET_CAPABILITY, GET_STATUS만 구현 (핵심)
- **Phase 2**: 다른 GET_* 메시지 (필요시)
- **Phase 3**: STT_/STP_/RTS_ (C++ 호환성 필요시)

**현재는 구현하지 않고, 필요성이 명확해질 때 추가하는 것을 권장합니다.**
