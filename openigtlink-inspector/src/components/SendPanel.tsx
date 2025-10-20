import { useAppStore } from "../store/appStore";
import "./SendPanel.css";

interface SendPanelProps {
  isExpanded: boolean;
  onToggle: () => void;
  showToSelector: boolean;
  tabId: number;
}

export default function SendPanel({
  isExpanded,
  onToggle,
  showToSelector,
  tabId,
}: SendPanelProps) {
  const { sendPanel, updateSendPanel, sendMessage } = useAppStore();
  const isSending = false; // 나중에 로딩 상태 추가 가능

  const handleSendOnce = async () => {
    if (!sendPanel.messageType || !sendPanel.deviceName) {
      alert("Please fill in all required fields");
      return;
    }

    try {
      await sendMessage(
        tabId,
        sendPanel.messageType,
        sendPanel.deviceName,
        sendPanel.content,
      );
      // 전송 성공 후 폼 초기화 선택 사항
      // updateSendPanel({ content: '' })
    } catch (error) {
      console.error("Send failed:", error);
    }
  };

  const handleRepeat = () => {
    updateSendPanel({ isRepeating: !sendPanel.isRepeating });
    // 실제 반복 전송 로직은 나중에 구현
  };

  return (
    <div className="send-panel">
      <div className="send-panel-header">
        <button className="expand-btn" onClick={onToggle}>
          {isExpanded ? "▼" : "▶"} Send Message
        </button>

        {!isExpanded && (
          <>
            <select
              className="quick-select"
              value={sendPanel.messageType}
              onChange={(e) => updateSendPanel({ messageType: e.target.value })}
              disabled={isSending}
            >
              <option>TRANSFORM</option>
              <option>STATUS</option>
              <option>STRING</option>
              <option>IMAGE</option>
              <option>SENSOR</option>
            </select>

            <input
              type="text"
              className="quick-input"
              value={sendPanel.deviceName}
              onChange={(e) => updateSendPanel({ deviceName: e.target.value })}
              placeholder="Device name"
              disabled={isSending}
            />

            {showToSelector && (
              <>
                <label>To:</label>
                <select
                  className="quick-select"
                  value={sendPanel.toClient}
                  onChange={(e) =>
                    updateSendPanel({ toClient: e.target.value })
                  }
                  style={{ minWidth: "120px" }}
                  disabled={isSending}
                >
                  <option>All Clients</option>
                  <option>Client-1</option>
                  <option>Client-2</option>
                </select>
              </>
            )}

            <button
              className="send-btn"
              onClick={handleSendOnce}
              disabled={isSending}
            >
              {isSending ? "Sending..." : "Send"}
            </button>
          </>
        )}
      </div>

      {isExpanded && (
        <div className="send-panel-expanded">
          {showToSelector && (
            <div className="form-group">
              <label>Send to:</label>
              <select
                value={sendPanel.toClient}
                onChange={(e) => updateSendPanel({ toClient: e.target.value })}
                disabled={isSending}
              >
                <option>All Clients</option>
                <option>Client-1</option>
                <option>Client-2</option>
              </select>
            </div>
          )}

          <div className="form-group">
            <label>Message Type:</label>
            <select
              value={sendPanel.messageType}
              onChange={(e) => updateSendPanel({ messageType: e.target.value })}
              disabled={isSending}
            >
              <option>TRANSFORM</option>
              <option>STATUS</option>
              <option>STRING</option>
              <option>IMAGE</option>
              <option>SENSOR</option>
              <option>POSITION</option>
              <option>QTDATA</option>
            </select>
          </div>

          <div className="form-group">
            <label>Device Name:</label>
            <input
              type="text"
              value={sendPanel.deviceName}
              onChange={(e) => updateSendPanel({ deviceName: e.target.value })}
              disabled={isSending}
            />
          </div>

          <div className="form-group">
            <label>Content (JSON):</label>
            <textarea
              className="content-editor"
              placeholder='Enter message content as JSON (e.g., {"string": "Hello"})'
              rows={5}
              value={sendPanel.content}
              onChange={(e) => updateSendPanel({ content: e.target.value })}
              disabled={isSending}
            />
          </div>

          <div className="button-group">
            <button
              className="send-btn"
              onClick={handleSendOnce}
              disabled={isSending}
            >
              {isSending ? "Sending..." : "Send Once"}
            </button>
            <button
              className={
                sendPanel.isRepeating ? "repeat-btn active" : "repeat-btn"
              }
              onClick={handleRepeat}
              disabled={isSending}
            >
              {sendPanel.isRepeating
                ? `Sending @ ${sendPanel.repeatRate}Hz (click to stop)`
                : `Send @ ${sendPanel.repeatRate}Hz`}
            </button>
            <button
              className="stop-btn"
              disabled={!sendPanel.isRepeating || isSending}
              onClick={() => updateSendPanel({ isRepeating: false })}
            >
              Stop
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
