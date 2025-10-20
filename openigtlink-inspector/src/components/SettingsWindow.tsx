import { useAppStore } from "../store/appStore";
import "./SettingsWindow.css";

interface SettingsWindowProps {
  onClose: () => void;
}

export default function SettingsWindow({ onClose }: SettingsWindowProps) {
  const {
    settings,
    setTheme,
    setBufferSize,
    setAutoScroll,
    setAutoReconnect,
    setVerifyCrc,
  } = useAppStore();

  return (
    <div className="settings-overlay">
      <div className="settings-window">
        <div className="settings-header">
          <h2>⚙️ Settings</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="settings-content">
          <div className="settings-section">
            <h3>Appearance</h3>
            <div className="setting-item">
              <label htmlFor="theme">Theme:</label>
              <select
                id="theme"
                value={settings.theme}
                onChange={(e) =>
                  setTheme(e.target.value as "dark" | "light" | "auto")
                }
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
                value={settings.bufferSize}
                onChange={(e) => setBufferSize(parseInt(e.target.value))}
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
                  checked={settings.autoScroll}
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
                  checked={settings.autoReconnect}
                  onChange={(e) => setAutoReconnect(e.target.checked)}
                />
                Auto-reconnect on disconnect
              </label>
            </div>
            <div className="setting-item">
              <label htmlFor="verifyCrc">
                <input
                  id="verifyCrc"
                  type="checkbox"
                  checked={settings.verifyCrc}
                  onChange={(e) => setVerifyCrc(e.target.checked)}
                />
                Verify CRC
              </label>
            </div>
          </div>
        </div>

        <div className="settings-footer">
          <button className="save-btn" onClick={onClose}>
            Save & Close
          </button>
        </div>
      </div>
    </div>
  );
}
