import { useState } from 'react'
import './SendPanel.css'

interface SendPanelProps {
  isExpanded: boolean
  onToggle: () => void
  showToSelector: boolean
}

export default function SendPanel({
  isExpanded,
  onToggle,
  showToSelector,
}: SendPanelProps) {
  const [messageType, setMessageType] = useState('TRANSFORM')
  const [deviceName, setDeviceName] = useState('TestDevice')
  const [toClient, setToClient] = useState('All Clients')

  return (
    <div className="send-panel">
      <div className="send-panel-header">
        <button
          className="expand-btn"
          onClick={onToggle}
        >
          {isExpanded ? '▼' : '▶'} Send Message
        </button>

        {!isExpanded && (
          <>
            <select
              className="quick-select"
              value={messageType}
              onChange={(e) => setMessageType(e.target.value)}
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
              value={deviceName}
              onChange={(e) => setDeviceName(e.target.value)}
              placeholder="Device name"
            />

            {showToSelector && (
              <>
                <label>To:</label>
                <select
                  className="quick-select"
                  value={toClient}
                  onChange={(e) => setToClient(e.target.value)}
                  style={{ minWidth: '120px' }}
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
                value={toClient}
                onChange={(e) => setToClient(e.target.value)}
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
              value={messageType}
              onChange={(e) => setMessageType(e.target.value)}
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
              value={deviceName}
              onChange={(e) => setDeviceName(e.target.value)}
            />
          </div>

          <div className="form-group">
            <label>Content:</label>
            <textarea
              className="content-editor"
              placeholder="Enter message content..."
              rows={5}
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
  )
}
