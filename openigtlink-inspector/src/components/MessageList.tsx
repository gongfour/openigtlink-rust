import { ReceivedMessage } from '../App'
import './MessageList.css'

interface MessageListProps {
  messages: ReceivedMessage[]
}

export default function MessageList({ messages }: MessageListProps) {
  const getMessageColor = (type: string): string => {
    const colors: { [key: string]: string } = {
      'TRANSFORM': '#4caf50',
      'IMAGE': '#2196f3',
      'STATUS': '#ff9800',
      'STRING': '#9c27b0',
      'SENSOR': '#00bcd4',
      'POSITION': '#f44336',
      'QTDATA': '#8bc34a',
    }
    return colors[type] || '#888'
  }

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString('ko-KR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    })
  }

  return (
    <div className="message-list">
      <table className="message-table">
        <thead>
          <tr>
            <th style={{ width: '120px' }}>Timestamp</th>
            <th style={{ width: '100px' }}>Type</th>
            <th style={{ width: '150px' }}>Device</th>
            <th style={{ width: '80px' }}>Size (bytes)</th>
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
              <tr key={index} className="message-row">
                <td>{formatTimestamp(msg.timestamp)}</td>
                <td>
                  <span
                    className="type-badge"
                    style={{
                      backgroundColor: getMessageColor(msg.message_type),
                      color: 'white'
                    }}
                  >
                    {msg.message_type}
                  </span>
                </td>
                <td className="device-cell">{msg.device_name}</td>
                <td className="size-cell">{msg.size_bytes}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  )
}
