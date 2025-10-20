# OpenIGTLink Inspector - 상태관리 리팩토링 종합 가이드

## 📊 핵심 요약

### 현재 상황
```
✅ 완료: Zustand 도입, 기본 전역 상태관리
⚠️ 문제: 로컬 상태 분산, 탭별 메시지 미분리, 기능 미구현
```

### 해결 방안
```
1. 로컬 상태 → 전역 상태 통합
2. 탭별 메시지 독립적 관리
3. 기능별 상태 분리
4. 백엔드 구조 개선
```

---

## 🎯 주요 문제점과 해결책

### 문제 1: 분산된 UI 상태
```
❌ 현재
SendPanel.tsx
├── messageType: useState ← 로컬
├── deviceName: useState ← 로컬
└── toClient: useState ← 로컬

MessageList.tsx
└── expandedKeys: useState ← 로컬

✅ 개선 후
Zustand Store
├── sendPanel: { messageType, deviceName, toClient } ← 전역
└── messageList: { expandedKeys } ← 전역
```

**영향도:** 높음 | **난이도:** 낮음 | **예상 시간:** 4-6시간

---

### 문제 2: 탭별 메시지 미분리
```
❌ 현재
messages: ReceivedMessage[] (모든 탭이 공유)
├── 탭 A의 메시지
├── 탭 B의 메시지 ← 섞여있음!
└── 탭 C의 메시지

✅ 개선 후
tabMessages: Map<number, ReceivedMessage[]>
├── 1: [탭 1의 메시지만]
├── 2: [탭 2의 메시지만]
└── 3: [탭 3의 메시지만]
```

**영향도:** 최고 | **난이도:** 중상 | **예상 시간:** 6-8시간

**왜 중요한가?**
- 각 탭이 독립적인 연결을 가져야 함
- 탭별 메시지 필터링 필요
- 탭 전환 시에도 상태 유지

---

### 문제 3: 백엔드 단일 연결 관리
```rust
❌ 현재
pub struct ConnectionManager {
    client: Option<ClientHandle>,  // 하나만 가능
}

✅ 개선 후
pub struct ConnectionManager {
    connections: HashMap<usize, TabConnection>,
    // 각 탭별 독립적 연결
}
```

**영향도:** 높음 | **난이도:** 중상 | **예상 시간:** 4-6시간

---

### 문제 4: 메시지 전송 기능 미구현
```
❌ 현재: SendPanel UI만 있음
✅ 개선 후: 
  1. 폼 데이터 수집 (Zustand)
  2. Tauri 커맨드 호출
  3. 백엔드에서 메시지 생성 및 전송
  4. 전송 결과 피드백
```

**영향도:** 중상 | **난이도:** 중상 | **예상 시간:** 8-10시간

---

## 📋 구체적인 구현 단계

### 1단계: SendPanel 상태 통합 (4-6시간)

#### 1.1 appStore.ts 수정
```typescript
// 추가할 상태
interface SendPanelState {
  messageType: string;
  deviceName: string;
  toClient: string;
  content: string;
  isRepeating: boolean;
  repeatRate: number;
}

interface AppState {
  // ... 기존
  sendPanel: SendPanelState;
  
  // 액션 추가
  updateSendPanel(data: Partial<SendPanelState>): void;
  clearSendPanel(): void;
}

// 구현
updateSendPanel: (data) => set((state) => ({
  sendPanel: { ...state.sendPanel, ...data }
})),

clearSendPanel: () => set((state) => ({
  sendPanel: {
    messageType: 'TRANSFORM',
    deviceName: 'TestDevice',
    toClient: 'All Clients',
    content: '',
    isRepeating: false,
    repeatRate: 60,
  }
})),
```

#### 1.2 SendPanel.tsx 수정
```typescript
import { useAppStore } from '../store/appStore'

export default function SendPanel({ ... }) {
  const { sendPanel, updateSendPanel } = useAppStore()

  return (
    <div>
      <select
        value={sendPanel.messageType}
        onChange={(e) => updateSendPanel({ messageType: e.target.value })}
      >
        {/* ... */}
      </select>
      {/* ... */}
    </div>
  )
}
```

#### 1.3 MessageList.tsx 수정
```typescript
// expandedKeys를 Zustand로 이동
// ... 유사하게 구현
```

---

### 2단계: 탭별 메시지 분리 (6-8시간)

#### 2.1 appStore.ts 수정
```typescript
interface AppState {
  // 변경: 전역 messages → 탭별 messages
  tabMessages: Map<number, ReceivedMessage[]>;
  
  // 새 액션들
  addTabMessage(tabId: number, message: ReceivedMessage): void;
  setTabMessages(tabId: number, messages: ReceivedMessage[]): void;
  clearTabMessages(tabId: number): void;
}

// 구현 예시
addTabMessage: (tabId, message) => set((state) => {
  const messages = state.tabMessages.get(tabId) || [];
  const updated = [message, ...messages].slice(0, 1000);
  const newMap = new Map(state.tabMessages);
  newMap.set(tabId, updated);
  return { tabMessages: newMap };
}),
```

#### 2.2 App.tsx 수정
```typescript
useEffect(() => {
  const unlisten = listen("message_received", (event: any) => {
    const { tabId, message } = event.payload;
    // 변경: 탭별로 메시지 저장
    addTabMessage(tabId, message);
  });
  return () => { unlisten.then((fn) => fn()); };
}, [addTabMessage]);
```

#### 2.3 MessagePanel.tsx 수정
```typescript
interface MessagePanelProps {
  tab: Tab;
  tabId: number;  // NEW
  // messages 제거
  // ...
}

export default function MessagePanel({ tab, tabId, ... }) {
  const messages = useAppStore((state) => 
    state.tabMessages.get(tabId) || []
  );
  
  return (
    <MessageList messages={messages} />
  );
}
```

---

### 3단계: 메시지 전송 기능 (8-10시간)

#### 3.1 백엔드: commands.rs 추가
```rust
#[tauri::command]
pub async fn send_message(
    tab_id: usize,
    message_type: String,
    device_name: String,
    content: serde_json::Value,
    connection: State<'_, Mutex<ConnectionManager>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let conn = connection.lock().map_err(|e| e.to_string())?;
    
    // 탭별 연결 확인
    let tab_conn = conn.get_connection(tab_id)
        .ok_or("Tab not connected")?;
    
    // 메시지 생성
    let message = create_message(&message_type, &device_name, &content)?;
    
    // 메시지 전송
    tab_conn.client.send(message).await?;
    
    // 성공 이벤트 발송
    app.emit_all("message_sent", json!({
        "tab_id": tab_id,
        "success": true
    }))?;
    
    Ok(())
}

fn create_message(
    msg_type: &str,
    device_name: &str,
    content: &serde_json::Value,
) -> Result<AnyMessage, String> {
    // 메시지 타입별로 생성
    match msg_type {
        "TRANSFORM" => {
            let matrix = parse_matrix(content)?;
            Ok(AnyMessage::Transform(...))
        },
        "STATUS" => { /* ... */ },
        // ... 등등
        _ => Err(format!("Unknown message type: {}", msg_type)),
    }
}
```

#### 3.2 프론트엔드: appStore.ts
```typescript
interface SentMessage {
  id: string;
  timestamp: number;
  messageType: string;
  deviceName: string;
  success: boolean;
  error?: string;
}

interface AppState {
  sentMessages: SentMessage[];
  
  sendMessage(tabId: number, message: SendPanelState): Promise<void>;
  addSentMessage(message: SentMessage): void;
}

// 구현
sendMessage: async (tabId, message) => {
  try {
    await invoke("send_message", {
      tab_id: tabId,
      message_type: message.messageType,
      device_name: message.deviceName,
      content: parseContent(message.content),
    });
    
    set((state) => ({
      sentMessages: [...state.sentMessages, {
        id: uuid(),
        timestamp: Date.now(),
        messageType: message.messageType,
        deviceName: message.deviceName,
        success: true,
      }],
    }));
  } catch (error) {
    set((state) => ({
      sentMessages: [...state.sentMessages, {
        id: uuid(),
        timestamp: Date.now(),
        messageType: message.messageType,
        deviceName: message.deviceName,
        success: false,
        error: error.message,
      }],
    }));
  }
},
```

#### 3.3 프론트엔드: SendPanel.tsx
```typescript
export default function SendPanel({ ... }) {
  const { sendPanel, updateSendPanel, activeTab } = useAppStore();
  const sendMessage = useAppStore((state) => state.sendMessage);
  
  const handleSendOnce = async () => {
    await sendMessage(activeTab, sendPanel);
    updateSendPanel({ content: '' });  // 폼 초기화
  };
  
  const handleRepeat = async () => {
    updateSendPanel({ isRepeating: true });
    // 반복 전송 로직...
  };
  
  return (
    <div>
      {/* ... */}
      <button onClick={handleSendOnce}>Send Once</button>
      <button onClick={handleRepeat}>Send @ {sendPanel.repeatRate}Hz</button>
    </div>
  );
}
```

---

### 4단계: 필터링 기능 (4-6시간)

```typescript
interface FilterState {
  searchText: string;
  selectedTypes: string[];
  selectedDevices: string[];
  timeRange: { start: number; end: number } | null;
}

// appStore.ts에 추가
filters: FilterState;

setSearchText(text: string): void;
setMessageTypeFilter(types: string[]): void;
setDeviceNameFilter(devices: string[]): void;
setTimeRange(range: { start: number; end: number } | null): void;
clearFilters(): void;

// 필터링된 메시지 가져오기 (selector)
getFilteredMessages(tabId: number): ReceivedMessage[] {
  const messages = this.tabMessages.get(tabId) || [];
  const { filters } = this;
  
  return messages.filter(msg => {
    if (filters.searchText && 
        !msg.device_name.includes(filters.searchText) &&
        !msg.message_type.includes(filters.searchText)) {
      return false;
    }
    
    if (filters.selectedTypes.length > 0 &&
        !filters.selectedTypes.includes(msg.message_type)) {
      return false;
    }
    
    // ... 나머지 필터 로직
    
    return true;
  });
}
```

---

### 5단계: 로깅/내보내기 (4-6시간)

```typescript
interface LoggingState {
  isLogging: boolean;
  logFilePath: string | null;
  exportFormat: 'json' | 'csv' | 'txt';
}

// appStore.ts에 추가
logging: LoggingState;

startLogging(filePath: string): Promise<void>;
stopLogging(): void;
exportMessages(tabId: number, format: string): Promise<string>;

// 구현
startLogging: async (filePath) => {
  const result = await invoke("start_logging", { file_path: filePath });
  set({ logging: { isLogging: true, logFilePath: filePath, ... } });
},

exportMessages: async (tabId, format) => {
  const messages = get().tabMessages.get(tabId) || [];
  const result = await invoke("export_messages", {
    messages,
    format,
  });
  return result;
},
```

---

## 📈 상태 변화 흐름도

### Before
```
SendPanel 입력
  ↓
setState (로컬)
  ↓ (Props로 전달? 아니면 직접 invoke?)
불명확한 전송 방식
  ↓
메시지 모두에 브로드캐스트
  ↓
모든 탭 영향 ❌
```

### After
```
SendPanel 입력
  ↓
useAppStore().updateSendPanel()
  ↓
useAppStore().sendMessage()
  ↓
invoke("send_message", {tabId, ...})
  ↓
백엔드 처리
  ↓
emit_all("message_sent", {tabId, ...})
  ↓
useAppStore().addSentMessage()
  ↓
해당 탭만 업데이트 ✅
```

---

## 🔄 동시성 처리 고려사항

### 여러 탭에서 동시에 메시지 수신
```typescript
// Map으로 관리하면 각 탭이 독립적으로 업데이트
addTabMessage: (tabId, message) => set((state) => {
  const messages = state.tabMessages.get(tabId) || [];
  const updated = [message, ...messages].slice(0, 1000);
  
  const newMap = new Map(state.tabMessages);
  newMap.set(tabId, updated);
  
  return { tabMessages: newMap };
}),
```

### 여러 탭에서 동시에 메시지 전송
```typescript
// Promise.all로 여러 메시지 전송 가능
const results = await Promise.all([
  store.sendMessage(tab1, msg1),
  store.sendMessage(tab2, msg2),
  store.sendMessage(tab3, msg3),
]);
```

---

## 🧪 테스트 전략

### 단위 테스트
```typescript
describe('appStore', () => {
  it('should add message to specific tab', () => {
    const store = useAppStore();
    const msg = { ... };
    
    store.addTabMessage(1, msg);
    
    expect(store.tabMessages.get(1)).toContain(msg);
    expect(store.tabMessages.get(2)).toBeUndefined();
  });
  
  it('should update sendPanel state', () => {
    store.updateSendPanel({ messageType: 'STATUS' });
    expect(store.sendPanel.messageType).toBe('STATUS');
  });
});
```

### 통합 테스트
```typescript
describe('Message Send Flow', () => {
  it('should send message and update store', async () => {
    // 1. 폼 작성
    store.updateSendPanel({ messageType: 'TRANSFORM' });
    
    // 2. 메시지 전송
    await store.sendMessage(1, store.sendPanel);
    
    // 3. 전송 기록 확인
    expect(store.sentMessages.length).toBe(1);
    expect(store.sentMessages[0].success).toBe(true);
  });
});
```

---

## ✅ 완료 후 확인사항

- [ ] 모든 로컬 상태가 Zustand로 통합됨
- [ ] 각 탭이 독립적인 메시지 관리
- [ ] SendPanel에서 메시지 전송 가능
- [ ] 필터링 기능 동작
- [ ] 로깅/내보내기 기능 동작
- [ ] 타입 에러 없음
- [ ] 빌드 성공
- [ ] 기본 사용성 테스트 완료

---

## 🚀 다음 단계

1. **Phase 1 시작:** SendPanel 상태 통합
2. **테스트:** 각 Phase마다 빌드/테스트 실행
3. **커밋:** Phase별로 커밋
4. **코드 리뷰:** 변경사항 검토
5. **배포:** 모든 Phase 완료 후 main 병합

이 가이드를 따르면 체계적이고 유지보수 가능한 상태관리 구조를 만들 수 있습니다!
