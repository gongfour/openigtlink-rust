import { useAppStore } from "../store/appStore";
import "./SendPanel.css";

interface SendPanelProps {
  isExpanded: boolean;
  onToggle: () => void;
  showToSelector: boolean;
}

export default function SendPanel({
  isExpanded,
  onToggle,
  showToSelector,
}: SendPanelProps) {
  const { sendPanel, updateSendPanel } = useAppStore();

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
                >
                  <option>All Clients</option>
                  <option>Client-1</option>
                  <option>Client-2</option>
                </select>
              </>
            )}

            <button className="send-btn">Send</button>
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
            />
          </div>

          <div className="form-group">
            <label>Content:</label>
            <textarea
              className="content-editor"
              placeholder="Enter message content..."
              rows={5}
              value={sendPanel.content}
              onChange={(e) => updateSendPanel({ content: e.target.value })}
            />
          </div>

          <div className="button-group">
            <button className="send-btn">Send Once</button>
            <button className="repeat-btn">Send @ 60Hz</button>
            <button className="stop-btn">Stop</button>
          </div>
        </div>
      )}
    </div>
  );
}
