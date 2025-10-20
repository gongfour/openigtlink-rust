import './TopBar.css'

interface TopBarProps {
  onSettingsClick: () => void
}

export default function TopBar({ onSettingsClick }: TopBarProps) {
  return (
    <div className="top-bar">
      <div className="left-section">
        <h1>🔍 OpenIGTLink Inspector</h1>
      </div>
      <div className="right-section">
        <button
          className="icon-button"
          onClick={onSettingsClick}
          title="Settings"
        >
          ⚙️ Settings
        </button>
        <button
          className="icon-button"
          title="Toggle Theme"
        >
          🌙 Theme
        </button>
      </div>
    </div>
  )
}
