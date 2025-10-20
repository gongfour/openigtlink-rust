import { useState } from 'react'
import './SettingsWindow.css'

interface SettingsWindowProps {
  onClose: () => void
}

export default function SettingsWindow({ onClose }: SettingsWindowProps) {
  const [theme, setTheme] = useState('dark')
  const [bufferSize, setBufferSize] = useState('1000')
  const [autoScroll, setAutoScroll] = useState(true)

  return (
    <div className="settings-overlay">
      <div className="settings-window">
        <div className="settings-header">
          <h2>⚙️ Settings</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="settings-content">
          <div className="settings-section">
            <h3>Appearance</h3>
            <div className="setting-item">
              <label htmlFor="theme">Theme:</label>
              <select
                id="theme"
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
                <option value="auto">Auto</option>
              </select>
            </div>
          </div>

          <div className="settings-section">
            <h3>Messages</h3>
            <div className="setting-item">
              <label htmlFor="buffer">Buffer Size:</label>
              <input
                id="buffer"
                type="number"
                value={bufferSize}
                onChange={(e) => setBufferSize(e.target.value)}
                min="100"
                max="10000"
                step="100"
              />
            </div>
            <div className="setting-item">
              <label htmlFor="autoscroll">
                <input
                  id="autoscroll"
                  type="checkbox"
                  checked={autoScroll}
                  onChange={(e) => setAutoScroll(e.target.checked)}
                />
                Auto-scroll to latest
              </label>
            </div>
          </div>

          <div className="settings-section">
            <h3>Connection</h3>
            <div className="setting-item">
              <label htmlFor="autoReconnect">
                <input
                  id="autoReconnect"
                  type="checkbox"
                  defaultChecked={false}
                />
                Auto-reconnect on disconnect
              </label>
            </div>
            <div className="setting-item">
              <label htmlFor="verifyCrc">
                <input
                  id="verifyCrc"
                  type="checkbox"
                  defaultChecked={true}
                />
                Verify CRC
              </label>
            </div>
          </div>
        </div>

        <div className="settings-footer">
          <button className="save-btn" onClick={onClose}>Save & Close</button>
        </div>
      </div>
    </div>
  )
}
