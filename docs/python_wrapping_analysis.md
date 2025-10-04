# Python Wrapping 시 스트리밍 메시지 필요성 분석

## 🐍 시나리오: Rust → Python Binding

### 1. **Python에서 사용하는 패턴**

#### 기존 Python OpenIGTLink (pyigtl)
```python
import pyigtl

# 패턴 1: Blocking wait (Pull)
client = pyigtl.OpenIGTLinkClient("127.0.0.1", 18944)
message = client.wait_for_message("ToolToReference", timeout=5)
print(message)

# 패턴 2: Continuous polling
while True:
    message = client.get_latest_message("Tracker")
    if message:
        process(message)
    time.sleep(0.01)
```

### 2. **Rust → Python Binding 시 가능한 패턴**

#### Option A: 현재 방식 (Pull 기반)
```python
# PyO3 binding
import openigtlink_rust

client = openigtlink_rust.Client.connect("127.0.0.1:18944")

# 방법 1: 수동 루프
while True:
    msg = client.receive_transform()
    process(msg)

# 방법 2: Generator
for msg in client.iter_messages():
    process(msg)
```

#### Option B: 스트리밍 제어 (Push 기반)
```python
import openigtlink_rust

client = openigtlink_rust.Client.connect("127.0.0.1:18944")

# 1. 스트리밍 시작 요청
client.start_streaming("TDATA", fps=60)

# 2. Callback 방식
def on_tracking_data(msg):
    print(f"Received: {msg}")

client.on_message("TDATA", on_tracking_data)

# 3. 자동으로 계속 수신...

# 4. 중지
client.stop_streaming("TDATA")
```

### 3. **Python 특성과 스트리밍 제어**

#### ❌ Python GIL 문제
```python
# Python의 Global Interpreter Lock
# → 한 번에 하나의 스레드만 Python 코드 실행

# 문제: 백그라운드 스레드에서 메시지 수신해도
# Python callback 호출 시 GIL 대기 필요
def callback(msg):  # ← GIL 필요
    print(msg)      # ← Python 객체 접근

# Rust 스레드가 Python 호출 시 병목
```

#### ✅ 해결 방법들

**1. AsyncIO 패턴**
```python
import asyncio
import openigtlink_rust

async def receive_loop():
    client = openigtlink_rust.AsyncClient.connect("127.0.0.1:18944")

    # Rust의 Tokio와 Python asyncio 통합
    async for msg in client.stream_tracking():
        await process(msg)

asyncio.run(receive_loop())
```

**2. Queue 패턴**
```python
from queue import Queue
import threading

# Rust → Python Queue (GIL 최소화)
msg_queue = Queue()

def rust_receiver_thread():
    while True:
        msg = client.receive()  # Rust code (no GIL)
        msg_queue.put(msg)      # Thread-safe

# Python main thread
while True:
    msg = msg_queue.get()
    process(msg)
```

**3. Iterator 패턴 (가장 Pythonic)**
```python
# Blocking iterator
for msg in client.iter_tracking_data():
    process(msg)

# Non-blocking iterator
for msg in client.iter_tracking_data(timeout=0.1):
    if msg:
        process(msg)
```

### 4. **스트리밍 제어가 필요한 경우**

#### ✅ **필요함 - C++ 서버 연동**
```python
# Slicer, PLUS Toolkit 등 C++ OpenIGTLink 서버와 통신
import openigtlink_rust

client = openigtlink_rust.Client.connect("slicer.server:18944")

# C++ 서버는 STT_TDATA를 기다림
client.send_start_tracking("TDATA", fps=60)  # ← 필요!

# 서버가 자동으로 TDATA 푸시
for msg in client.iter_messages():
    print(msg)

client.send_stop_tracking("TDATA")
```

#### ❌ **불필요 - Rust 서버 연동**
```python
# Rust 서버는 클라이언트 요청에 즉시 응답
import openigtlink_rust

client = openigtlink_rust.Client.connect("rust.server:18944")

# 직접 요청
for i in range(100):
    msg = client.request_tracking_data()  # ← 간단
    process(msg)
```

### 5. **Python Binding 설계 권장안**

#### Phase 1: 기본 패턴 (Pull)
```python
# PyO3 binding
from openigtlink_rust import Client, TransformMessage

client = Client.connect("127.0.0.1:18944")

# 1. 단일 송수신
msg = TransformMessage.identity()
client.send(msg)
response = client.receive()

# 2. Iterator
for msg in client.iter_messages(timeout=1.0):
    process(msg)

# 3. AsyncIO
async for msg in client.stream_messages():
    await process(msg)
```

#### Phase 2: 스트리밍 제어 추가 (C++ 호환)
```python
# C++ OpenIGTLink 서버와 통신 시 필요
client = Client.connect("slicer:18944")

# 스트리밍 시작
client.start_streaming(
    message_type="TDATA",
    device_name="Tracker",
    fps=60
)

# 자동 수신
for msg in client.iter_messages():
    if msg.type == "TDATA":
        print(msg.tracking_data)

# 스트리밍 중지
client.stop_streaming("TDATA")
```

### 6. **API 설계 예시**

```python
class Client:
    """OpenIGTLink Client (Rust-backed)"""

    # === Phase 1: 기본 기능 ===

    def send(self, message: Message) -> None:
        """메시지 전송"""
        pass

    def receive(self, timeout: float = None) -> Message:
        """메시지 수신 (blocking)"""
        pass

    def iter_messages(self, timeout: float = None):
        """메시지 스트림 (generator)"""
        while True:
            yield self.receive(timeout)

    # === Phase 2: 스트리밍 제어 (C++ 호환) ===

    def start_streaming(
        self,
        message_type: str,
        device_name: str = "",
        fps: int = 60,
        coordinate: str = "Patient"
    ) -> None:
        """스트리밍 시작 요청 (STT_ 전송)"""
        pass

    def stop_streaming(
        self,
        message_type: str,
        device_name: str = ""
    ) -> None:
        """스트리밍 중지 요청 (STP_ 전송)"""
        pass

    # === Phase 3: AsyncIO ===

    async def async_receive(self) -> Message:
        """비동기 수신"""
        pass

    async def stream_messages(self):
        """비동기 스트림"""
        while True:
            yield await self.async_receive()
```

### 7. **실제 사용 시나리오**

#### 시나리오 A: 3D Slicer 연동
```python
# Slicer는 C++ OpenIGTLink 서버
import openigtlink_rust

client = openigtlink_rust.Client.connect("localhost:18944")

# 1. Capability 조회
caps = client.get_capability()
print(f"Slicer supports: {caps.types}")

# 2. Transform 스트리밍 시작
client.start_streaming("TRANSFORM", device_name="NeedleTip", fps=30)

# 3. 실시간 수신
for transform in client.iter_messages():
    if transform.type == "TRANSFORM":
        print(f"Needle position: {transform.position}")

        # 목표 도달 시 중지
        if reached_target(transform.position):
            client.stop_streaming("TRANSFORM")
            break
```

#### 시나리오 B: 순수 Python 애플리케이션
```python
# Rust 서버 또는 간단한 통신
import openigtlink_rust

server = openigtlink_rust.Server.bind("0.0.0.0:18944")

# Pull 기반으로 충분
while True:
    msg = server.receive()

    if msg.type == "STRING":
        if msg.content == "GET_POSITION":
            response = get_current_position()
            server.send(response)
```

### 8. **결론**

**Python wrapping 시 스트리밍 제어 메시지:**

| 상황 | STT/STP 필요성 | 이유 |
|-----|---------------|------|
| **C++ OpenIGTLink 연동** | ✅ **필수** | Slicer, PLUS 등 C++ 서버 표준 |
| **Rust ↔ Python** | ❌ 불필요 | Iterator/AsyncIO로 충분 |
| **의료기기 인증** | ✅ **필수** | 표준 프로토콜 준수 |
| **프로토타입** | ❌ 불필요 | 간단한 패턴 선호 |

**권장 구현 순서:**

1. **Phase 1**: 기본 send/receive + iterator (필수)
2. **Phase 2**: AsyncIO 지원 (Python다운 API)
3. **Phase 3**: STT/STP 스트리밍 제어 (C++ 호환 필요 시)

**현재 상태:**
- Rust 구현체에 STT/STP 없어도 Python binding 가능
- C++ OpenIGTLink 서버와 통신 시에만 필요
- 우선순위: 기본 API → AsyncIO → 스트리밍 제어
