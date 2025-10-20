import { Tab } from '../App'
import './StatusBar.css'

interface StatusBarProps {
  tab: Tab | null
}

export default function StatusBar({ tab }: StatusBarProps) {
  if (!tab) {
    return (
      <div className="status-bar">
        <span className="status-text">No tab selected</span>
      </div>
    )
  }

  const status = tab.is_connected
    ? tab.tab_type === 'Client'
      ? `Connected to ${tab.host}:${tab.port}`
      : `Listening on :${tab.port} | Clients: 0`
    : 'Disconnected'

  return (
    <div className="status-bar">
      <span className="status-text">{status}</span>
      <div className="status-separator"></div>
      <span className="status-stat">Rx: {tab.rx_count} msgs</span>
      <div className="status-separator"></div>
      <span className="status-stat">Tx: {tab.tx_count} msgs</span>
    </div>
  )
}
