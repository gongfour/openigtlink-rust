# 상태관리 개선 계획서

## 현재 상태 (As-Is)

### 문제점 다이어그램
```
현재 상태 분산:

App.tsx (Zustand로 관리)
├── tabs ✓
├── activeTab ✓
├── messages (전체 메시지) ⚠️
├── settings ✓
└── showSettings ✓

MessageList.tsx (로컬 상태) ❌
├── expandedKeys (로컬 useState)

SendPanel.tsx (로컬 상태) ❌
├── messageType (로컬 useState)
├── deviceName (로컬 useState)
└── toClient (로컬 useState)

Tauri Backend ⚠️
└── ConnectionManager (단일 인스턴스)
    └── 모든 탭이 공유
```

**문제:**
1. 메시지가 탭별로 분리되지 않음 (모든 탭이 같은 메시지 배열 공유)
2. SendPanel의 폼 데이터가 로컬 상태 (전역 접근 불가)
3. MessageList의 UI 상태가 로컬 상태 (다른 탭으로 전환해도 상태 유지 안 됨)
4. 백엔드에서 각 탭별 독립적 연결 관리 불가

---

## 목표 상태 (To-Be)

### 개선된 상태 구조
```
Zustand AppStore (모든 상태 중앙화)
│
├── Tab Management
│   ├── tabs: Tab[]
│   ├── activeTab: number
│   ├── nextTabId: number
│   └── operations: addTab, removeTab, updateTab
│
├── Per-Tab State (NEW)
│   └── tabStates: Map<tabId, TabState>
│       ├── TabState {
│       │   messages: ReceivedMessage[]
│       │   expandedMessageKeys: Set<string>
│       │   sendPanel: SendPanelState
│       │   filters: FilterState
│       │   connectionState: ConnectionState
│       └── }
│
├── Send/Message Features (NEW)
│   ├── sendHistory: SentMessage[]
│   ├── messageTemplates: MessageTemplate[]
│   └── operations: sendMessage, saveTemplate
│
├── Filter/Search Features (NEW)
│   ├── searchText: string
│   ├── selectedMessageTypes: string[]
│   └── timeRange: TimeRange | null
│
├── Logging/Export Features (NEW)
│   ├── isLogging: boolean
│   ├── logFilePath: string | null
│   └── exportFormat: 'json' | 'csv'
│
└── Settings & UI
    ├── settings: Settings
    ├── showSettings: boolean
    └── operations: updateSettings
```

---

## Phase별 구현 계획

### Phase 1: 로컬 상태 → 전역 상태 (1-2일)
**목표:** 모든 UI 상태를 Zustand로 통합

```typescript
// 추가할 상태
interface SendPanelState {
  messageType: string;
  deviceName: string;
  toClient: string;
  selectedContent: string;
  isRepeating: boolean;
  repeatRate: number;
}

interface TabState {
  expandedMessageKeys: Set<string>;
  sendPanel: SendPanelState;
}

// 추가할 액션
updateSendPanel(tabId: number, data: Partial<SendPanelState>): void;
toggleMessageExpanded(tabId: number, messageKey: string): void;
```

**변경 파일:**
- `src/store/appStore.ts` - 상태/액션 추가
- `src/components/SendPanel.tsx` - Zustand 훅 사용
- `src/components/MessageList.tsx` - Zustand 훅 사용

---

### Phase 2: 탭별 메시지 분리 (2-3일)
**목표:** 각 탭이 자신의 메시지만 관리

```typescript
// 변경 전
messages: ReceivedMessage[]  // 모든 탭이 공유

// 변경 후
tabMessages: Map<number, ReceivedMessage[]>  // 탭별 독립 관리

// 사용
const messages = store.tabMessages.get(activeTab) || [];
```

**변경 파일:**
- `src/store/appStore.ts` - 메시지 구조 변경
- `src/App.tsx` - 메시지 수신 시 tabId 기반 저장
- `src/components/MessagePanel.tsx` - 탭별 메시지 사용

**백엔드 변경:**
- `src-tauri/src/commands.rs` - tabId 파라미터 추가
- `src-tauri/src/connection.rs` - 탭별 연결 관리

---

### Phase 3: 메시지 전송 기능 (2-3일)
**목표:** SendPanel에서 실제 메시지 전송

```typescript
// 추가할 액션
sendMessage(tabId: number, message: OutgoingMessage): Promise<void>;
addSentMessage(tabId: number, message: SentMessage): void;

// 백엔드 커맨드 추가
#[tauri::command]
async fn send_message(
    tab_id: usize,
    message_type: String,
    device_name: String,
    content: serde_json::Value,
) -> Result<(), String> { ... }
```

**변경 파일:**
- `src/store/appStore.ts` - sendMessage 액션
- `src/components/SendPanel.tsx` - 전송 로직 구현
- `src-tauri/src/commands.rs` - send_message 커맨드
- `src-tauri/src/connection.rs` - 메시지 전송 로직

---

### Phase 4: 필터링/검색 기능 (1-2일)
**목표:** 메시지 필터링 및 검색

```typescript
interface FilterState {
  searchText: string;
  selectedTypes: string[];
  selectedDevices: string[];
  timeRange: { start: number; end: number } | null;
}

// 추가할 액션
setSearchText(text: string): void;
setMessageTypeFilter(types: string[]): void;
clearFilters(): void;

// Selector (필터링된 메시지)
getFilteredMessages(tabId: number): ReceivedMessage[];
```

**변경 파일:**
- `src/store/appStore.ts` - 필터 상태/액션 추가
- `src/components/MessageList.tsx` - 필터 표시 UI
- `src/components/MessagePanel.tsx` - 필터 컨트롤

---

### Phase 5: 로깅/내보내기 (1-2일)
**목표:** 메시지 저장 및 내보내기

```typescript
// 추가할 상태
logging: {
  isLogging: boolean;
  logFilePath: string | null;
  exportFormat: 'json' | 'csv' | 'txt';
}

// 추가할 액션
startLogging(filePath: string): Promise<void>;
stopLogging(): void;
exportMessages(tabId: number, format: string): Promise<void>;
```

**백엔드 커맨드:**
```rust
#[tauri::command]
async fn export_messages(messages: Vec<ReceivedMessage>, format: String) -> Result<String, String> { ... }

#[tauri::command]
async fn start_logging(file_path: String) -> Result<(), String> { ... }
```

---

## 데이터 흐름 비교

### Before (현재)
```
User Input (SendPanel)
    ↓
setState() [로컬]
    ↓ (수동으로 invoke 호출 필요)
invoke("send_message")
    ↓
Backend 처리
    ↓
emit_all("message_received")
    ↓
App.tsx에서 수신
    ↓
setMessages() [전역]
    ↓
모든 탭이 메시지 받음 (❌ 문제)
```

### After (개선 후)
```
User Input (SendPanel)
    ↓
useAppStore().sendMessage(tabId, message)
    ↓ (자동으로 invoke 호출)
Backend 처리
    ↓
emit_all("message_received", {tabId, message})
    ↓
App.tsx에서 수신
    ↓
useAppStore().addTabMessage(tabId, message)
    ↓
해당 탭만 메시지 받음 (✓ 개선)
```

---

## 파일 구조 개선 계획

### 현재
```
src/
├── App.tsx
├── store/
│   └── appStore.ts
└── components/
    ├── [컴포넌트들]
```

### 개선 후 (권장)
```
src/
├── App.tsx
├── store/
│   ├── appStore.ts           (기존)
│   ├── types.ts              (NEW: 상태 타입 정의)
│   ├── hooks/
│   │   ├── useTabState.ts   (NEW: 탭별 상태 훅)
│   │   ├── useSendPanel.ts  (NEW: SendPanel 훅)
│   │   └── useFilters.ts    (NEW: 필터 훅)
│   └── slices/              (NEW: 기능별 액션 분리)
│       ├── tabSlice.ts
│       ├── messageSlice.ts
│       ├── sendSlice.ts
│       └── filterSlice.ts
├── services/
│   ├── api.ts               (NEW: Tauri 커맨드 래퍼)
│   └── messageParser.ts     (NEW: 메시지 파싱)
├── utils/
│   ├── filters.ts           (NEW: 필터링 유틸)
│   ├── format.ts            (NEW: 포맷팅 유틸)
│   └── validation.ts        (NEW: 검증 유틸)
└── components/
    ├── [기존 컴포넌트]
    └── [신규 컴포넌트들]
```

---

## 구현 체크리스트

### Phase 1: 로컬 상태 통합
- [ ] SendPanelState 타입 정의
- [ ] appStore에 sendPanel 상태 추가
- [ ] SendPanel.tsx에서 Zustand 사용
- [ ] MessageList 상태를 Zustand로 이동
- [ ] 빌드/테스트

### Phase 2: 탭별 메시지 분리
- [ ] TabState 타입 정의
- [ ] appStore에 tabMessages 추가
- [ ] App.tsx의 메시지 수신 로직 변경
- [ ] MessagePanel에서 탭별 메시지 사용
- [ ] 빌드/테스트

### Phase 3: 메시지 전송
- [ ] SentMessage 타입 정의
- [ ] appStore에 sendMessage 액션 추가
- [ ] send_message Tauri 커맨드 구현
- [ ] SendPanel에서 전송 로직 구현
- [ ] 백엔드 메시지 직렬화 구현
- [ ] 빌드/테스트

### Phase 4: 필터링
- [ ] FilterState 타입 정의
- [ ] 필터 액션 구현
- [ ] 필터링 유틸 함수 작성
- [ ] UI 필터 컨트롤 추가
- [ ] 빌드/테스트

### Phase 5: 로깅/내보내기
- [ ] LoggingState 타입 정의
- [ ] 로깅 액션 구현
- [ ] export_messages Tauri 커맨드
- [ ] 파일 저장 로직
- [ ] 빌드/테스트

---

## 예상 효과

### 개선 전
- 탭 간 메시지 공유로 인한 혼동
- 폼 상태 추적 어려움
- 새 기능 추가 시 상태관리 복잡
- 확장성 제한

### 개선 후
- ✅ 각 탭이 독립적인 메시지 관리
- ✅ 모든 상태가 중앙화된 store에서 관리
- ✅ 새 기능 추가 시 store에만 추가
- ✅ 높은 확장성 및 유지보수성
- ✅ 상태 디버깅 용이 (Redux DevTools 연동 가능)

---

## 일정 추정

| Phase | 예상 소요 시간 | 난이도 |
|-------|---------------|--------|
| 1: 로컬 상태 통합 | 4-6시간 | 하 |
| 2: 탁별 메시지 분리 | 6-8시간 | 중 |
| 3: 메시지 전송 | 8-10시간 | 중상 |
| 4: 필터링 | 4-6시간 | 중 |
| 5: 로깅/내보내기 | 4-6시간 | 하중 |
| **Total** | **26-36시간** | - |

---

## 결론

현재 구조는 좋은 기초를 제공하지만, 다음 개선사항이 필요합니다:

1. **로컬 상태 → 전역 상태** (즉시)
2. **탭별 메시지 독립 관리** (중요)
3. **기능별 상태 분리** (구조화)
4. **백엔드와의 타입 안전성** (필수)

이 계획을 따르면 유지보수성과 확장성이 크게 개선될 것입니다.
