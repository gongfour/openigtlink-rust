import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import TopBar from './components/TopBar'
import TabBar from './components/TabBar'
import MessagePanel from './components/MessagePanel'
import StatusBar from './components/StatusBar'
import SettingsWindow from './components/SettingsWindow'
import './App.css'

export interface Tab {
  id: number
  name: string
  tab_type: 'Client' | 'Server'
  host: string
  port: string
  is_connected: boolean
  send_panel_expanded: boolean
  rx_count: number
  tx_count: number
  error_message?: string
}

export interface ReceivedMessage {
  timestamp: number
  message_type: string
  device_name: string
  size_bytes: number
  from_client?: string
  body: string
}

function App() {
  const [tabs, setTabs] = useState<Tab[]>([
    {
      id: 0,
      name: 'Client-0',
      tab_type: 'Client',
      host: '127.0.0.1',
      port: '18944',
      is_connected: false,
      send_panel_expanded: false,
      rx_count: 0,
      tx_count: 0,
    }
  ])
  const [activeTab, setActiveTab] = useState(0)
  const [nextTabId, setNextTabId] = useState(1)
  const [showNewTabDialog, setShowNewTabDialog] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [messages, setMessages] = useState<ReceivedMessage[]>([])

  useEffect(() => {
    // Listen for new messages from backend
    const unlisten = listen('message_received', (event: any) => {
      const message = event.payload as ReceivedMessage
      setMessages(prev => {
        const updated = [message, ...prev]
        // Keep only last 1000 messages
        return updated.slice(0, 1000)
      })

      // Update rx_count
      setTabs(prev => {
        const updated = [...prev]
        if (updated[activeTab]) {
          updated[activeTab].rx_count += 1
        }
        return updated
      })
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [activeTab])

  const handleConnectClient = async (tabIndex: number) => {
    const tab = tabs[tabIndex]
    try {
      await invoke('connect_client', {
        host: tab.host,
        port: parseInt(tab.port),
      })

      const updated = [...tabs]
      updated[tabIndex].is_connected = true
      setTabs(updated)
    } catch (error) {
      const updated = [...tabs]
      updated[tabIndex].error_message = `Connection failed: ${error}`
      setTabs(updated)
    }
  }

  const handleDisconnect = async (tabIndex: number) => {
    try {
      await invoke('disconnect_client')
      const updated = [...tabs]
      updated[tabIndex].is_connected = false
      setTabs(updated)
    } catch (error) {
      console.error('Disconnect failed:', error)
    }
  }

  const handleAddTab = (type: 'Client' | 'Server') => {
    const newTab: Tab = {
      id: nextTabId,
      name: type === 'Client' ? `Client-${nextTabId}` : `Server-${nextTabId}`,
      tab_type: type,
      host: type === 'Client' ? '127.0.0.1' : '',
      port: '18944',
      is_connected: false,
      send_panel_expanded: false,
      rx_count: 0,
      tx_count: 0,
    }

    setTabs([...tabs, newTab])
    setActiveTab(tabs.length)
    setNextTabId(nextTabId + 1)
    setShowNewTabDialog(false)
  }

  const handleRemoveTab = (index: number) => {
    const updated = tabs.filter((_, i) => i !== index)
    setTabs(updated)
    if (activeTab >= updated.length && activeTab > 0) {
      setActiveTab(activeTab - 1)
    }
  }

  const handleUpdateTab = (index: number, changes: Partial<Tab>) => {
    const updated = [...tabs]
    updated[index] = { ...updated[index], ...changes }
    setTabs(updated)
  }

  return (
    <div className="app">
      <TopBar onSettingsClick={() => setShowSettings(!showSettings)} />

      <TabBar
        tabs={tabs}
        activeTab={activeTab}
        onTabClick={setActiveTab}
        onTabClose={handleRemoveTab}
        onAddTab={() => setShowNewTabDialog(true)}
      />

      <div className="main-content">
        {activeTab < tabs.length ? (
          <MessagePanel
            tab={tabs[activeTab]}
            messages={messages}
            onTabChange={(changes) => handleUpdateTab(activeTab, changes)}
            onConnect={() => handleConnectClient(activeTab)}
            onDisconnect={() => handleDisconnect(activeTab)}
          />
        ) : (
          <div className="empty-state">
            <p>No tabs open. Click '+ New Tab' to create one.</p>
          </div>
        )}
      </div>

      <StatusBar
        tab={activeTab < tabs.length ? tabs[activeTab] : null}
      />

      {showNewTabDialog && (
        <div className="dialog-overlay">
          <div className="dialog">
            <h2>New Connection</h2>
            <p>Select connection type:</p>
            <div className="dialog-buttons">
              <button onClick={() => handleAddTab('Client')}>📡 Client Connection</button>
              <button onClick={() => handleAddTab('Server')}>🏠 Server (Listen)</button>
            </div>
            <button className="cancel-btn" onClick={() => setShowNewTabDialog(false)}>Cancel</button>
          </div>
        </div>
      )}

      {showSettings && (
        <SettingsWindow onClose={() => setShowSettings(false)} />
      )}
    </div>
  )
}

export default App
