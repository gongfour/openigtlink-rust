# OpenIGTLink CLI 도구 설계 문서

## 목차
1. [개요](#개요)
2. [명령어 구조](#명령어-구조)
3. [옵션 정의](#옵션-정의)
4. [사용 예제](#사용-예제)
5. [메시지 포맷](#메시지-포맷)
6. [테스트 전략](#테스트-전략)
7. [구현 로드맵](#구현-로드맵)

---

## 개요

### 목적
OpenIGTLink Rust 구현이 C++ 구현과 호환 가능한지 검증하고,
프로토콜 메시지를 쉽게 송수신할 수 있는 CLI 도구 제공

### 주요 기능
- **Server 모드**: 클라이언트 연결 수락, 메시지 송수신
- **Client 모드**: 서버에 연결, 메시지 송수신
- **모든 메시지 타입 지원**: TRANSFORM, IMAGE, STATUS, SENSOR 등 21가지
- **유연한 설정**: 반복, 간격, 타임아웃 등 커스터마이징

---

## 명령어 구조

### 기본 구조
```
openigtlink <COMMAND> [OPTIONS]
```

### 지원 명령어

#### 1. SERVER 명령어
```bash
openigtlink server [OPTIONS]
```
서버 모드로 클라이언트 연결 수락 및 메시지 송수신

#### 2. CLIENT 명령어
```bash
openigtlink client [OPTIONS]
```
클라이언트 모드로 서버에 연결하여 메시지 송수신

#### 3. INFO 명령어 (향후)
```bash
openigtlink info [OPTIONS]
```
지원하는 메시지 타입, 버전 정보 등 조회

---

## 옵션 정의

### 바인딩/연결 옵션

#### Server용
```
--listen <ADDR:PORT>
  설명: 바인드할 주소와 포트
  기본값: 0.0.0.0:18944
  예시: --listen 127.0.0.1:18944
```

#### Client용
```
--connect <HOST:PORT>
  설명: 연결할 서버 주소와 포트
  필수: Yes
  예시: --connect localhost:18944
```

---

### SEND (메시지 전송) 옵션

```
--send-enable
  설명: 메시지 전송 기능 활성화
  타입: boolean flag
  필수: No (기본값: false)

--send-message-file <PATH>
  설명: 전송할 메시지가 저장된 JSON 파일 경로
  타입: file path
  필수: --send-enable일 때 필수
  예시: --send-message-file message.json

--send-repeat-count <N>
  설명: 메시지를 몇 번 반복 전송할지
  타입: integer
  기본값: 1
  범위: 1 - 100000
  예시: --send-repeat-count 100

--send-interval-ms <MS>
  설명: 반복 전송 간격 (밀리초)
  타입: integer
  기본값: 1000
  범위: 1 - 60000
  예시: --send-interval-ms 10

--send-device-name <NAME>
  설명: 메시지의 디바이스 이름 (JSON에서 없을 경우 사용)
  타입: string
  필수: No
  예시: --send-device-name "Tool1"
```

---

### RECEIVE (메시지 수신) 옵션

```
--receive-enable
  설명: 메시지 수신 기능 활성화
  타입: boolean flag
  필수: No (기본값: false)

--receive-message-types <TYPE1,TYPE2,...>
  설명: 받을 메시지 타입 (쉼표로 구분)
  타입: comma-separated string
  기본값: all
  가능한 값:
    - all (모든 메시지 타입)
    - TRANSFORM, IMAGE, STATUS, SENSOR, COMMAND, STRING
    - POSITION, QTDATA, TDATA, POINT, POLYDATA, TRAJECTORY
    - VIDEO, BIND, CAPABILITY, IMGMETA, VIDEOMETA, LBMETA, COLORTABLE
  예시: --receive-message-types TRANSFORM,IMAGE,STATUS

--receive-max-count <N>
  설명: 최대 몇 개의 메시지까지 받을지
  타입: integer
  기본값: 0 (무한)
  범위: 0 - 100000
  예시: --receive-max-count 10

--receive-timeout-sec <SECS>
  설명: 메시지 수신 대기 시간 (초)
  타입: integer
  기본값: 30
  범위: 1 - 3600
  예시: --receive-timeout-sec 60

--receive-output-file <PATH>
  설명: 받은 메시지를 저장할 파일 경로
  타입: file path
  필수: No
  예시: --receive-output-file messages.json

--receive-output-format <FORMAT>
  설명: 출력 형식
  타입: enum
  기본값: json
  가능한 값: json, raw, text
  예시: --receive-output-format json
```

---

### 공통 옵션

```
--log-level <LEVEL>
  설명: 로그 수준
  타입: enum
  기본값: info
  가능한 값: debug, info, warn, error
  예시: --log-level debug

--log-file <PATH>
  설명: 로그를 파일에 저장
  타입: file path
  필수: No
  예시: --log-file app.log

--help, -h
  설명: 도움말 표시
  타입: boolean flag

--version, -v
  설명: 버전 정보 표시
  타입: boolean flag
```

---

## 사용 예제

### 기본 예제

#### 예제 1: 서버 시작 (기본)
```bash
openigtlink server
# 0.0.0.0:18944에서 대기
```

#### 예제 2: 클라이언트 연결 (기본)
```bash
openigtlink client --connect localhost:18944
# localhost:18944에 연결하고 대기
```

---

### SEND 예제

#### 예제 3: 서버에서 메시지 전송 (1회)
```bash
openigtlink server \
  --listen 0.0.0.0:18944 \
  --send-enable \
  --send-message-file transform.json
```

#### 예제 4: 클라이언트에서 메시지 100개 전송 (10ms 간격)
```bash
openigtlink client \
  --connect localhost:18944 \
  --send-enable \
  --send-message-file transform.json \
  --send-repeat-count 100 \
  --send-interval-ms 10
```

#### 예제 5: 다양한 메시지 타입 전송
```bash
# TRANSFORM 전송
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file examples/transform.json \
  --send-repeat-count 50

# IMAGE 전송
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file examples/image.json \
  --send-repeat-count 10

# STATUS 전송
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file examples/status.json \
  --send-repeat-count 5
```

---

### RECEIVE 예제

#### 예제 6: 서버에서 모든 메시지 받기
```bash
openigtlink server \
  --listen 0.0.0.0:18944 \
  --receive-enable \
  --receive-message-types all \
  --receive-max-count 0 \
  --receive-timeout-sec 60 \
  --receive-output-file received.json
```

#### 예제 7: 클라이언트에서 특정 타입 메시지 받기 (최대 10개, 30초)
```bash
openigtlink client \
  --connect localhost:18944 \
  --receive-enable \
  --receive-message-types TRANSFORM,IMAGE \
  --receive-max-count 10 \
  --receive-timeout-sec 30 \
  --receive-output-file response.json
```

#### 예제 8: 클라이언트에서 IMAGE만 받기 (무한)
```bash
openigtlink client \
  --connect localhost:18944 \
  --receive-enable \
  --receive-message-types IMAGE \
  --receive-max-count 0 \
  --receive-timeout-sec 10 \
  --log-level debug
```

---

### 양방향 통신 예제

#### 예제 9: 서버가 메시지를 받고 응답 전송
```bash
# 터미널 1: 서버 - 메시지 받고 응답 보내기
openigtlink server \
  --listen 0.0.0.0:18944 \
  --receive-enable \
  --receive-message-types TRANSFORM \
  --receive-max-count 1 \
  --receive-timeout-sec 30 \
  --receive-output-file received.json \
  --send-enable \
  --send-message-file examples/ack.json \
  --send-repeat-count 1

# 터미널 2: 클라이언트 - 메시지 보내고 응답 받기
openigtlink client \
  --connect localhost:18944 \
  --send-enable \
  --send-message-file examples/transform.json \
  --send-repeat-count 1 \
  --receive-enable \
  --receive-message-types STATUS \
  --receive-max-count 1 \
  --receive-timeout-sec 10 \
  --receive-output-file response.json
```

---

### 디버깅 예제

#### 예제 10: DEBUG 로그로 서버 실행
```bash
openigtlink server \
  --listen 0.0.0.0:18944 \
  --log-level debug \
  --log-file server.log \
  --receive-enable \
  --receive-message-types all
```

---

## 메시지 포맷

### JSON 메시지 파일 구조

#### TRANSFORM 메시지
```json
{
  "device_name": "Tool1",
  "timestamp": 1234567890,
  "transform": {
    "matrix": [
      [1.0, 0.0, 0.0, 100.0],
      [0.0, 1.0, 0.0, 50.0],
      [0.0, 0.0, 1.0, 200.0],
      [0.0, 0.0, 0.0, 1.0]
    ]
  }
}
```

#### IMAGE 메시지
```json
{
  "device_name": "Camera",
  "timestamp": 1234567890,
  "image": {
    "size": [512, 512, 1],
    "scalar_type": "uint8",
    "spacing": [1.0, 1.0, 1.0],
    "origin": [0.0, 0.0, 0.0],
    "data_base64": "base64_encoded_image_data_here"
  }
}
```

#### STATUS 메시지
```json
{
  "device_name": "Server",
  "timestamp": 1234567890,
  "status": {
    "code": 1,
    "message": "Success",
    "sub_code": 0
  }
}
```

#### SENSOR 메시지
```json
{
  "device_name": "ForceSensor",
  "timestamp": 1234567890,
  "sensor": {
    "unit": 1,
    "readings": [1.2, -0.5, 3.8, 0.1, 0.05, -0.2]
  }
}
```

#### POSITION 메시지
```json
{
  "device_name": "Marker",
  "timestamp": 1234567890,
  "position": {
    "x": 100.0,
    "y": 50.0,
    "z": 200.0
  }
}
```

---

## 테스트 전략

### 1. 단위 테스트

#### 1.1 메시지 파싱 테스트
- JSON → 메시지 타입 변환 테스트
- 메시지 타입 → JSON 변환 테스트

#### 1.2 옵션 파싱 테스트
- 유효한 옵션 조합 테스트
- 무효한 옵션 조합 테스트

---

### 2. 통합 테스트

#### 2.1 기본 송수신 테스트
```bash
# 테스트 1: 단순 TRANSFORM 송수신
# 1. 서버 시작
# 2. 클라이언트가 메시지 전송
# 3. 서버가 수신 확인
# ✓ PASS/FAIL

# 테스트 2: 다양한 메시지 타입
for TYPE in TRANSFORM IMAGE STATUS SENSOR; do
  # 서버에서 TYPE 메시지 받기
  # 클라이언트에서 TYPE 메시지 보내기
  # ✓ PASS/FAIL
done
```

#### 2.2 성능 테스트
```bash
# 테스트 3: 높은 주파수 (100Hz)
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file transform.json \
  --send-repeat-count 1000 \
  --send-interval-ms 10
# 검증: 모든 메시지가 성공적으로 전송되었는가?

# 테스트 4: 대용량 데이터 (IMAGE)
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file large_image.json \
  --send-repeat-count 100
# 검증: 데이터 무결성 확인
```

---

### 3. 호환성 테스트 (C++ 구현)

#### 3.1 Rust 서버 + C++ 클라이언트
```bash
# 1. Rust 서버 시작
openigtlink server --listen 127.0.0.1:18944 \
  --receive-enable \
  --receive-message-types all \
  --receive-output-file rust_received.json

# 2. C++ 클라이언트에서 메시지 전송
# 3. Rust 서버가 메시지 수신 확인
# ✓ 메시지 형식 일치 확인
```

#### 3.2 C++ 서버 + Rust 클라이언트
```bash
# 1. C++ 서버 시작
# 2. Rust 클라이언트 연결
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file transform.json \
  --receive-enable \
  --receive-message-types all

# ✓ 성공 여부 확인
```

---

### 4. 테스트 시나리오

#### 시나리오 A: 기본 TRANSFORM 송수신
```
Server: listen on 0.0.0.0:18944
Client: send TRANSFORM message → Server receives and validates
Expected: Message correctly received and formatted
```

#### 시나리오 B: 스트리밍 이미지 송수신
```
Server: listen and receive IMAGE messages
Client: send 10 IMAGE messages at 100ms intervals
Expected: All 10 images received without loss
```

#### 시나리오 C: 양방향 통신
```
Server: receive COMMAND, send STATUS response
Client: send COMMAND, receive STATUS response
Expected: Request-response pattern works correctly
```

---

## 구현 로드맵

### Phase 1: 기본 구조 (1주)
- [ ] `src/bin/openigtlink-cli.rs` 생성
- [ ] clap 라이브러리 추가
- [ ] 기본 명령어 파싱 (server, client)
- [ ] 바인딩/연결 기능 구현

**완료 조건:**
```bash
openigtlink server --listen 127.0.0.1:18944
openigtlink client --connect localhost:18944
```

---

### Phase 2: SEND 기능 (1주)
- [ ] `--send-enable` 구현
- [ ] `--send-message-file` 구현
- [ ] `--send-repeat-count` 구현
- [ ] `--send-interval-ms` 구현
- [ ] 메시지 파일 파싱 (JSON)

**완료 조건:**
```bash
openigtlink client --connect localhost:18944 \
  --send-enable \
  --send-message-file message.json \
  --send-repeat-count 10 \
  --send-interval-ms 100
```

---

### Phase 3: RECEIVE 기능 (1주)
- [ ] `--receive-enable` 구현
- [ ] `--receive-message-types` 구현
- [ ] `--receive-max-count` 구현
- [ ] `--receive-timeout-sec` 구현
- [ ] `--receive-output-file` 구현
- [ ] 메시지 저장 기능

**완료 조건:**
```bash
openigtlink server \
  --listen 0.0.0.0:18944 \
  --receive-enable \
  --receive-message-types TRANSFORM,IMAGE \
  --receive-max-count 10 \
  --receive-output-file messages.json
```

---

### Phase 4: 로깅 및 디버깅 (3일)
- [ ] `--log-level` 구현
- [ ] `--log-file` 구현
- [ ] 상세한 로그 메시지 추가

**완료 조건:**
```bash
openigtlink server --log-level debug --log-file app.log
```

---

### Phase 5: 테스트 및 문서 (1주)
- [ ] 단위 테스트 작성
- [ ] 통합 테스트 작성
- [ ] 호환성 테스트 실행
- [ ] 사용 설명서 작성
- [ ] 예제 메시지 파일 작성

**완료 조건:**
```bash
cargo test
./tests/integration_tests.sh
./tests/compatibility_tests.sh
```

---

### Phase 6: INFO 명령어 (향후)
- [ ] `openigtlink info --version` 구현
- [ ] `openigtlink info --supported-types` 구현
- [ ] 도움말 개선

---

## 예제 메시지 파일 구조

### `examples/messages/` 디렉토리
```
examples/messages/
├── transform.json
├── image.json
├── status.json
├── sensor.json
├── position.json
├── string.json
├── command.json
├── video.json
├── point.json
├── polydata.json
├── trajectory.json
├── qtdata.json
├── tdata.json
├── ndarray.json
├── bind.json
├── capability.json
├── imgmeta.json
├── videometa.json
├── lbmeta.json
├── colortable.json
└── large_image.json (테스트용 대용량)
```

---

## 부록: 옵션 요약 테이블

### SEND 옵션
| 옵션 | 타입 | 필수 | 기본값 | 설명 |
|-----|------|------|--------|------|
| `--send-enable` | flag | No | false | 전송 활성화 |
| `--send-message-file` | path | Yes* | - | 메시지 파일 |
| `--send-repeat-count` | int | No | 1 | 반복 횟수 |
| `--send-interval-ms` | int | No | 1000 | 반복 간격(ms) |
| `--send-device-name` | string | No | - | 디바이스명 |

*: `--send-enable`일 때 필수

### RECEIVE 옵션
| 옵션 | 타입 | 필수 | 기본값 | 설명 |
|-----|------|------|--------|------|
| `--receive-enable` | flag | No | false | 수신 활성화 |
| `--receive-message-types` | string | No | all | 수신 타입 |
| `--receive-max-count` | int | No | 0 | 최대 개수 |
| `--receive-timeout-sec` | int | No | 30 | 타임아웃(초) |
| `--receive-output-file` | path | No | - | 저장 파일 |
| `--receive-output-format` | enum | No | json | 출력 형식 |

---

## 참고사항

- 모든 경로는 상대경로 또는 절대경로 모두 가능
- 메시지 타입은 대소문자 구분 (TRANSFORM, not transform)
- 동시에 send와 receive 모두 활성화 가능
- 타임아웃은 초 단위, 간격은 밀리초 단위

---

**마지막 수정**: 2025-11-02
**버전**: 1.1 (TLS 제외, 설계 단계)
