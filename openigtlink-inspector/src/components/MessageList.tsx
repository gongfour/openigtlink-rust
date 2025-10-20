import { ReceivedMessage } from "../App";
import { useAppStore } from "../store/appStore";
import "./MessageList.css";

interface MessageListProps {
  messages: ReceivedMessage[];
}

export default function MessageList({ messages }: MessageListProps) {
  const { expandedMessageKeys, toggleMessageExpanded } = useAppStore();

  // 메시지의 고유 키 생성 (타임스탬프 + 장치명 + 타입)
  const getMessageKey = (index: number): string => {
    const msg = messages[index];
    return `${msg.timestamp}-${msg.device_name}-${msg.message_type}-${index}`;
  };

  const getMessageColor = (type: string): string => {
    const colors: { [key: string]: string } = {
      TRANSFORM: "#4caf50",
      IMAGE: "#2196f3",
      STATUS: "#ff9800",
      STRING: "#9c27b0",
      SENSOR: "#00bcd4",
      POSITION: "#f44336",
      QTDATA: "#8bc34a",
    };
    return colors[type] || "#888";
  };

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("ko-KR", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      fractionalSecondDigits: 3,
    });
  };

  const handleRowClick = (index: number) => {
    const key = getMessageKey(index);
    toggleMessageExpanded(key);
  };

  return (
    <div className="message-list">
      <table className="message-table">
        <thead>
          <tr>
            <th style={{ width: "120px" }}>Timestamp</th>
            <th style={{ width: "100px" }}>Type</th>
            <th style={{ width: "150px" }}>Device</th>
            <th style={{ width: "80px" }}>Size (bytes)</th>
          </tr>
        </thead>
        <tbody>
          {messages.length === 0 ? (
            <tr>
              <td colSpan={4} className="empty-message">
                No messages received yet
              </td>
            </tr>
          ) : (
            messages.map((msg, index) => (
              <>
                <tr
                  key={`row-${index}`}
                  className={`message-row ${expandedMessageKeys.has(getMessageKey(index)) ? "selected" : ""}`}
                  onClick={() => handleRowClick(index)}
                >
                  <td>{formatTimestamp(msg.timestamp)}</td>
                  <td>
                    <span
                      className="type-badge"
                      style={{
                        backgroundColor: getMessageColor(msg.message_type),
                        color: "white",
                      }}
                    >
                      {msg.message_type}
                    </span>
                  </td>
                  <td className="device-cell">{msg.device_name}</td>
                  <td className="size-cell">{msg.size_bytes}</td>
                </tr>
                {expandedMessageKeys.has(getMessageKey(index)) && (
                  <tr key={`detail-${index}`} className="message-detail-row">
                    <td colSpan={4}>
                      <div className="inline-message-details">
                        <div className="detail-row">
                          <span className="detail-label">Timestamp:</span>
                          <span className="detail-value">
                            {formatTimestamp(msg.timestamp)}
                          </span>
                        </div>
                        <div className="detail-row">
                          <span className="detail-label">Type:</span>
                          <span className="detail-value">
                            {msg.message_type}
                          </span>
                        </div>
                        <div className="detail-row">
                          <span className="detail-label">Device Name:</span>
                          <span className="detail-value">
                            {msg.device_name}
                          </span>
                        </div>
                        <div className="detail-row">
                          <span className="detail-label">Size:</span>
                          <span className="detail-value">
                            {msg.size_bytes} bytes
                          </span>
                        </div>
                        {msg.from_client && (
                          <div className="detail-row">
                            <span className="detail-label">From Client:</span>
                            <span className="detail-value">
                              {msg.from_client}
                            </span>
                          </div>
                        )}
                        <div className="detail-row">
                          <span className="detail-label">Body:</span>
                        </div>
                        <div className="body-content">
                          <pre>{JSON.stringify(msg.body, null, 2)}</pre>
                        </div>
                      </div>
                    </td>
                  </tr>
                )}
              </>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
