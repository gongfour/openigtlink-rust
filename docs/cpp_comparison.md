# C++ OpenIGTLink vs Rust 구현체 비교

## 📊 메시지 타입 비교

### 지원 메시지 현황

| 카테고리 | C++ OpenIGTLink | Rust 구현체 | 상태 |
|---------|----------------|-------------|------|
| **기본 메시지** | 15개 핵심 타입 | 20개 타입 | ✅ Rust가 5개 더 많음 |
| **쿼리 메시지** | 7개 (GET_*) | ❌ 미지원 | C++만 지원 |
| **스트리밍 제어** | 9개 (RTS_*, STT_*, STP_*) | ❌ 미지원 | C++만 지원 |
| **총계** | 34개 메시지 타입 | 20개 메시지 타입 | C++가 더 포괄적 |

### C++ 전용 메시지

**쿼리 메시지 (Query Messages):**
- GET_TRANS, GET_IMAGE, GET_STATUS, GET_POINT, GET_TRAJ
- GET_IMGMETA, GET_LBMETA, GET_POLYDATA

**스트리밍 제어 메시지:**
- RTS_* (Ready To Send): RTS_POLYDATA, RTS_TDATA, RTS_QTDATA, RTS_COMMAND
- STT_* (Start): STT_POLYDATA, STT_TDATA, STT_QTDATA
- STP_* (Stop): STP_POLYDATA, STP_TDATA, STP_QTDATA

### Rust 전용 메시지

- VIDEO, VIDEOMETA - 비디오 스트리밍
- SENSOR - 센서 데이터
- NDARRAY - N차원 배열
- COMMAND - 명령 메시지

## 🔧 예제 프로그램 비교

### C++ OpenIGTLink (20개 카테고리)

**의료 영상:**
- Imager (Server/Client 3종)
- ImageDatabaseServer
- ImageMeta

**추적 & 네비게이션:**
- Tracker, TrackingData, QuaternionTrackingData
- Point (Client/Server), PolyData

**통신 & 제어:**
- String, Status, Capability, Bind
- WebSocket, SessionManager

**기타:**
- Thread, Receiver, SampleUDPProgam
- TrackingDataUDPTransfer, Trajectory, VideoStreaming

### Rust 구현체 (13개)

**의료 영상:**
- image_streaming, video_streaming

**추적 & 네비게이션:**
- tracking_server, udp_tracking, point_navigation

**센서 & 데이터:**
- sensor_logger, ndarray_transfer

**통신:**
- string_command, status_monitor

**고급 기능:**
- async_server, error_handling, client, server

## 🧪 테스트 커버리지

### C++ OpenIGTLink (31개 테스트 파일)

- 메시지별 개별 테스트 (19개)
- 소켓 테스트 (3개)
- 멀티스레딩 테스트 (3개)
- RTP 래퍼, 타임스탬프 테스트

### Rust 구현체 (27개 테스트 모듈)

- 소스 파일 내 `#[cfg(test)]` 모듈
- 예제가 통합 테스트 역할

## 🎯 기술적 차별점

| 특징 | C++ | Rust |
|------|-----|------|
| 메모리 안전성 | 수동 | ✅ 컴파일러 보장 |
| 동시성 | pthread | ✅ Tokio async |
| 에러 처리 | 예외/반환코드 | ✅ Result<T,E> |
| 타입 안전성 | 런타임 | ✅ 컴파일타임 |
| 산업 검증 | ✅ 10+ 년 | 🆕 신규 |

## 📝 주요 기능 갭

**Rust 미지원:**
- GET_* 쿼리 메시지 (8개)
- RTS_*/STT_*/STP_* 스트리밍 제어 (9개)
- WebSocket, SessionManager, BIND

**C++ 미지원:**
- SENSOR, NDARRAY
- 에러 복구 패턴
- Tokio 비동기
