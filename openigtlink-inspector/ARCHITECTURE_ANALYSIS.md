# OpenIGTLink Inspector 상태관리 아키텍처 분석

## 1. 현재 구조 분석

### 1.1 프론트엔드 (React + TypeScript)
```
Frontend Structure:
├── App.tsx (루트 컴포넌트)
├── store/
│   └── appStore.ts (Zustand 전역 상태)
└── components/
    ├── TopBar.tsx
    ├── TabBar.tsx
    ├── MessagePanel.tsx
    ├── MessageList.tsx (로컬 상태: expandedKeys)
    ├── SendPanel.tsx (로컬 상태: messageType, deviceName, toClient)
    ├── StatusBar.tsx
    ├── SettingsWindow.tsx
    └── [다른 컴포넌트들]
```

### 1.2 백엔드 (Tauri + Rust)
```
Backend Structure:
├── main.rs (앱 초기화)
├── commands.rs (Tauri 커맨드)
│   ├── connect_client() - 클라이언트 연결
│   ├── disconnect_client() - 연결 해제
│   ├── get_connection_status() - 상태 조회
│   └── create_*_tab() - 탭 생성
├── connection.rs (ConnectionManager)
└── types.rs (데이터 구조)
```

---

## 2. 현재 상태관리 문제점

### 2.1 분산된 상태 관리
| 상태 | 위치 | 유형 |
|------|------|------|
| 탭 정보 | Zustand | 전역 ✓ |
| 메시지 | Zustand | 전역 ✓ |
| 설정 | Zustand | 전역 ✓ |
| MessageList 전개/축소 | MessageList.tsx | 로컬 ✗ |
| SendPanel 폼 데이터 | SendPanel.tsx | 로컬 ✗ |

**문제:**
- SendPanel의 폼 데이터(messageType, deviceName, toClient)가 로컬 상태
- MessageList의 expandedKeys도 로컬 상태
- Tauri 백엔드와의 상태 동기화 부족

### 2.2 탭별 독립적 상태 관리 부재
```
현재:
- 모든 탭이 동일한 messages 배열 공유
- 각 탭마다 별도의 메시지 필터링 필요
- 탭별 연결 상태 독립 관리 안 됨
```

### 2.3 Tauri 백엔드 상태 관리
```
문제점:
- ConnectionManager가 단 하나만 존재 (여러 탭 지원 불가)
- 각 탭별 독립적인 연결 관리 안 됨
- 메시지 수신이 모든 탭으로 브로드캐스트됨
```

---

## 3. 누락된 기능

### 3.1 전송 기능 (SendPanel)
- **상태:** 폼은 존재하나 실제 전송 기능 미구현
- **필요 상태:**
  - 메시지 타입, 장치명, 수신자 선택
  - 반복 전송 모드 (60Hz)
  - 전송 기록
  - 메시지 생성 템플릿

### 3.2 서버 모드 (Server Tab)
- **상태:** UI 틀만 있고 기능 미구현
- **필요 상태:**
  - 리스닝 포트
  - 연결된 클라이언트 목록
  - 각 클라이언트별 메시지 수신

### 3.3 메시지 필터링/검색
- **누락:** 메시지 검색, 필터링 기능 없음
- **필요 상태:**
  - 필터 조건 (메시지 타입, 장치명, 시간 범위)
  - 검색 텍스트

### 3.4 메시지 내보내기/로깅
- **누락:** 메시지 저장, 내보내기 기능 없음
- **필요 상태:**
  - 로그 파일 경로
  - 로깅 활성 여부
  - 내보내기 형식 설정

---

## 4. 개선된 상태관리 아키텍처 설계

### 4.1 새로운 Zustand Store 구조

```typescript
// 1. 탭별 상태 분리
interface TabState {
  id: number;
  messages: ReceivedMessage[];
  expandedMessageKeys: Set<string>;
  sendPanel: {
    messageType: string;
    deviceName: string;
    toClient: string;
    isRepeating: boolean;
    repeatRate: number;
  };
}

// 2. 전송 관련 상태
interface SendState {
  isSending: boolean;
  sendHistory: SentMessage[];
  messageTemplates: MessageTemplate[];
}

// 3. 검색/필터 상태
interface FilterState {
  searchText: string;
  messageTypeFilter: string[];
  deviceNameFilter: string[];
  timeRange: { start: number; end: number } | null;
}

// 4. 로깅/내보내기 상태
interface LoggingState {
  isLogging: boolean;
  logFilePath: string | null;
  exportFormat: 'json' | 'csv' | 'txt';
}

// 통합 스토어
interface AppState extends CurrentAppState {
  // 탭별 상태 (맵 구조)
  tabs: Map<number, TabState>;
  
  // 전역 상태
  send: SendState;
  filter: FilterState;
  logging: LoggingState;
  
  // 탭별 상태 액션들
  updateTabState: (tabId: number, state: Partial<TabState>) => void;
  addTabMessage: (tabId: number, message: ReceivedMessage) => void;
  toggleMessageExpanded: (tabId: number, messageKey: string) => void;
  updateSendPanel: (tabId: number, data: Partial<SendState['sendPanel']>) => void;
}
```

### 4.2 계층별 책임 분리

```
┌─────────────────────────────────────┐
│      React Components (UI Layer)     │
│  (TopBar, MessageList, SendPanel)   │
└──────────────┬──────────────────────┘
               │ useAppStore()
┌──────────────▼──────────────────────┐
│   Zustand Store (State Layer)        │
│  (전역 상태 + 액션)                   │
└──────────────┬──────────────────────┘
               │ invoke()
┌──────────────▼──────────────────────┐
│   Tauri Commands (API Layer)         │
│  (connect, disconnect, send)         │
└──────────────┬──────────────────────┘
               │ Tokio + Event Emit
┌──────────────▼──────────────────────┐
│   Rust Backend (Business Layer)      │
│  (ConnectionManager, Protocol)       │
└─────────────────────────────────────┘
```

### 4.3 탭별 독립적 연결 관리

```rust
// 백엔드 개선
pub struct TabConnection {
    pub id: usize,
    pub client: Option<ClientHandle>,
    pub is_connected: bool,
    pub rx_count: usize,
    pub tx_count: usize,
}

pub struct ConnectionManager {
    connections: HashMap<usize, TabConnection>,
}

// 각 탭별로 독립적인 스레드에서 메시지 수신
// 수신된 메시지는 탭 ID와 함께 프론트엔드로 전송
```

---

## 5. 상태 데이터 흐름

### 5.1 메시지 수신 흐름
```
Rust Backend
    ↓ (tokio::spawn로 메시지 수신)
emit_all("message_received", {tabId, message})
    ↓ (Tauri 이벤트)
Frontend listen("message_received")
    ↓ (React)
useAppStore().addTabMessage(tabId, message)
    ↓ (상태 업데이트)
MessageList 컴포넌트 리렌더링
```

### 5.2 메시지 전송 흐름
```
SendPanel 폼 제출
    ↓ (사용자 입력)
useAppStore().sendMessage()
    ↓ (store 액션)
invoke("send_message", {tabId, message})
    ↓ (Tauri 커맨드)
Rust 백엔드에서 메시지 전송
    ↓
emit_all("message_sent", {tabId, success})
    ↓
useAppStore().addSentMessage()
```

---

## 6. 리팩토링 전략

### Phase 1: 기초 (현재 진행 중)
- ✓ Zustand 도입
- ✓ 기본 상태관리 (탭, 메시지, 설정)
- [ ] SendPanel 상태를 Zustand로 이동
- [ ] MessageList expandedKeys를 Zustand로 이동

### Phase 2: 기능 확장
- [ ] 탭별 메시지 분리 (현재: 전역 메시지 배열)
- [ ] 탭별 메시지 필터링 상태
- [ ] 검색/필터 기능 구현
- [ ] 메시지 전송 기능 구현

### Phase 3: 백엔드 개선
- [ ] 각 탭별 독립적 ConnectionManager
- [ ] 탭 ID를 포함한 이벤트 시스템
- [ ] 서버 모드 구현
- [ ] 메시지 전송 기능

### Phase 4: 고급 기능
- [ ] 메시지 로깅 및 내보내기
- [ ] 메시지 템플릿 시스템
- [ ] 반복 전송 기능
- [ ] 메시지 히스토리

---

## 7. 구현 순서 (우선순위)

1. **높음 - SendPanel 상태 통합** (이번 주)
   - 로컬 상태 → Zustand
   - 전송 기능 구현 필요

2. **높음 - MessageList 상태 통합** (이번 주)
   - expandedKeys 로컬 상태 → Zustand
   - 탭별 메시지 분리 시작

3. **중간 - 탭별 메시지 분리** (다음 주)
   - messages 배열 → Map<tabId, messages[]>
   - 백엔드: 탭 ID와 함께 메시지 전송

4. **중간 - 기본 전송 기능** (다음 주)
   - send_message 커맨드 구현
   - 메시지 생성 및 직렬화

5. **낮음 - 고급 기능들** (이후)
   - 필터링, 로깅, 템플릿 등

---

## 8. 상세 개선 계획

### 8.1 현재 appStore.ts 확장
```typescript
// 추가할 상태
interface MessageFilterState {
  searchText: string;
  selectedTypes: string[];
  startTime: number | null;
  endTime: number | null;
}

interface SendState {
  isSending: boolean;
  sentMessages: SentMessage[];
  templates: MessageTemplate[];
}

// 탭별 상태 추가
tabs: Map<number, {
  id: number;
  // ... 기존 탭 정보
  messages: ReceivedMessage[];
  expandedMessageKeys: Set<string>;
  filter: MessageFilterState;
}>
```

### 8.2 새로운 커맨드 추가 (Rust)
```rust
#[tauri::command]
pub async fn send_message(
    tab_id: usize,
    message_type: String,
    device_name: String,
    content: serde_json::Value,
) -> Result<(), String> { ... }

#[tauri::command]
pub async fn start_server(
    tab_id: usize,
    port: u16,
) -> Result<(), String> { ... }

#[tauri::command]
pub fn get_connected_clients(tab_id: usize) -> Result<Vec<String>, String> { ... }
```

---

## 결론

현재 Zustand 기반의 상태관리는 좋은 기초입니다. 
다음 단계는:

1. **로컬 상태들을 전역 상태로 통합** (SendPanel, MessageList)
2. **탭별 메시지를 독립적으로 관리**
3. **백엔드에서 탭 ID 기반 커맨드 지원**
4. **메시지 전송, 필터링 등 기능 추가**

이 아키텍처를 따르면 향후 기능 추가 시에도 상태관리가 체계적으로 유지될 것입니다.
