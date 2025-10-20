import { Tab } from '../App'
import './TabBar.css'

interface TabBarProps {
  tabs: Tab[]
  activeTab: number
  onTabClick: (index: number) => void
  onTabClose: (index: number) => void
  onAddTab: () => void
}

export default function TabBar({
  tabs,
  activeTab,
  onTabClick,
  onTabClose,
  onAddTab,
}: TabBarProps) {
  return (
    <div className="tab-bar">
      <div className="tabs-container">
        {tabs.map((tab, index) => {
          const icon = tab.tab_type === 'Client' ? '📡' : '🏠'
          const statusColor = tab.is_connected ? '#4caf50' : '#888'

          return (
            <div
              key={tab.id}
              className={`tab ${index === activeTab ? 'active' : ''}`}
              onClick={() => onTabClick(index)}
            >
              <span className="tab-icon">{icon}</span>
              <span className="tab-name">{tab.name}</span>
              <span className="tab-status" style={{ color: statusColor }}>●</span>
              <button
                className="tab-close"
                onClick={(e) => {
                  e.stopPropagation()
                  onTabClose(index)
                }}
              >
                ×
              </button>
            </div>
          )
        })}
      </div>
      <div className="tab-actions">
        <button className="add-tab-btn" onClick={onAddTab}>
          + New Tab
        </button>
      </div>
    </div>
  )
}
