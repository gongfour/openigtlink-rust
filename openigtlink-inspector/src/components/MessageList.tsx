import { ReceivedMessage } from "../App";
import { useAppStore } from "../store/appStore";
import "./MessageList.css";

interface MessageListProps {
  messages: ReceivedMessage[];
  tabIndex: number;
}

export default function MessageList({ messages, tabIndex }: MessageListProps) {
  const tabs = useAppStore((state) => state.tabs);
  const toggleMessageExpanded = useAppStore((state) => state.toggleMessageExpanded);

  const expandedMessageKeys = tabs[tabIndex]?.expandedMessageKeys || new Set<string>();
  const isServerTab = tabs[tabIndex]?.tab_type === "Server";

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

  const getMessageData = (msg: ReceivedMessage): string => {
    const body = msg.body;

    switch (msg.message_type) {
      case "TRANSFORM":
        // 변환 행렬의 위치 정보 (translation)
        if (body.matrix?.data) {
          const tx = body.matrix.data[0][3];
          const ty = body.matrix.data[1][3];
          const tz = body.matrix.data[2][3];
          return `[${tx?.toFixed(2)}, ${ty?.toFixed(2)}, ${tz?.toFixed(2)}]`;
        }
        return "Transform matrix";

      case "POSITION":
        // 위치 데이터
        if (body.position) {
          const { x, y, z } = body.position;
          return `[${x?.toFixed(2)}, ${y?.toFixed(2)}, ${z?.toFixed(2)}]`;
        }
        return "Position data";

      case "STRING":
        // 문자열 데이터 (최대 30자)
        const str = body.string || "";
        return str.length > 30 ? str.substring(0, 30) + "..." : str;

      case "STATUS":
        // 상태 코드와 메시지
        return `${body.code || 0}: ${body.status_string || ""}`;

      case "IMAGE":
        // 이미지 크기
        if (body.size) {
          return `${body.size.x}×${body.size.y}×${body.size.z}`;
        }
        return "Image data";

      case "SENSOR":
        // 센서 데이터 길이
        return `${body.data_length || 0} samples`;

      case "CAPABILITY":
        // 지원하는 타입들
        if (body.types && Array.isArray(body.types)) {
          return body.types.join(", ");
        }
        return "Capability";

      default:
        return "—";
    }
  };

  const handleRowClick = (index: number) => {
    const key = getMessageKey(index);
    toggleMessageExpanded(tabIndex, key);
  };

  return (
    <div className="message-list">
      <table className="message-table">
        <thead>
          <tr>
            {isServerTab && <th style={{ width: "100px" }}>From Client</th>}
            <th style={{ width: "120px" }}>Timestamp</th>
            <th style={{ width: "100px" }}>Type</th>
            <th style={{ width: "150px" }}>Device</th>
            <th style={{ width: "200px" }}>Data</th>
            <th style={{ width: "80px" }}>Size (bytes)</th>
          </tr>
        </thead>
        <tbody>
          {messages.length === 0 ? (
            <tr>
              <td colSpan={isServerTab ? 6 : 5} className="empty-message">
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
                  {isServerTab && (
                    <td className="client-cell">
                      {msg.from_client || "Unknown"}
                    </td>
                  )}
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
                  <td className="data-cell">{getMessageData(msg)}</td>
                  <td className="size-cell">{msg.size_bytes}</td>
                </tr>
                {expandedMessageKeys.has(getMessageKey(index)) && (
                  <tr key={`detail-${index}`} className="message-detail-row">
                    <td colSpan={isServerTab ? 6 : 5}>
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
