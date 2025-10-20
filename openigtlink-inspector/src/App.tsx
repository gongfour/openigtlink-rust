import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import TopBar from "./components/TopBar";
import TabBar from "./components/TabBar";
import MessagePanel from "./components/MessagePanel";
import StatusBar from "./components/StatusBar";
import SettingsWindow from "./components/SettingsWindow";
import { useAppStore } from "./store/appStore";
import "./App.css";

export interface Tab {
  id: number;
  name: string;
  tab_type: "Client" | "Server";
  host: string;
  port: string;
  is_connected: boolean;
  send_panel_expanded: boolean;
  rx_count: number;
  tx_count: number;
  error_message?: string;
}

export interface ReceivedMessage {
  timestamp: number;
  message_type: string;
  device_name: string;
  size_bytes: number;
  from_client?: string;
  body: any; // JSON object containing message details
}

function App() {
  const {
    tabs,
    activeTab,
    nextTabId,
    showNewTabDialog,
    showSettings,
    setShowNewTabDialog,
    toggleSettings,
    addTabMessage,
    incrementRxCount,
    addTab,
    removeTab,
    updateTab,
    setTabConnected,
    setTabError,
    setActiveTab,
    setNextTabId,
  } = useAppStore();

  useEffect(() => {
    // Listen for new messages from backend
    const unlisten = listen("message_received", (event: any) => {
      const { tabId, message } = event.payload as {
        tabId: number;
        message: ReceivedMessage;
      };
      addTabMessage(tabId, message);

      // Update rx_count for the tab that received the message
      const tabIndex = tabs.findIndex((tab) => tab.id === tabId);
      if (tabIndex >= 0) {
        incrementRxCount(tabIndex);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [tabs, addTabMessage, incrementRxCount]);

  const handleConnectClient = async (tabIndex: number) => {
    const tab = tabs[tabIndex];
    try {
      await invoke("connect_client", {
        tab_id: tab.id,
        host: tab.host,
        port: parseInt(tab.port),
      });

      setTabConnected(tabIndex, true);
    } catch (error) {
      setTabError(tabIndex, `Connection failed: ${error}`);
    }
  };

  const handleDisconnect = async (tabIndex: number) => {
    try {
      await invoke("disconnect_client");
      setTabConnected(tabIndex, false);
    } catch (error) {
      console.error("Disconnect failed:", error);
    }
  };

  const handleAddTab = (type: "Client" | "Server") => {
    const newTab: Tab = {
      id: nextTabId,
      name: type === "Client" ? `Client-${nextTabId}` : `Server-${nextTabId}`,
      tab_type: type,
      host: type === "Client" ? "127.0.0.1" : "",
      port: "18944",
      is_connected: false,
      send_panel_expanded: false,
      rx_count: 0,
      tx_count: 0,
    };

    addTab(newTab);
    setNextTabId(nextTabId + 1);
    setShowNewTabDialog(false);
  };

  return (
    <div className="app">
      <TopBar onSettingsClick={toggleSettings} />

      <TabBar
        tabs={tabs}
        activeTab={activeTab}
        onTabClick={setActiveTab}
        onTabClose={removeTab}
        onAddTab={() => setShowNewTabDialog(true)}
      />

      <div className="main-content">
        {activeTab < tabs.length ? (
          <MessagePanel
            tab={tabs[activeTab]}
            onTabChange={(changes) => updateTab(activeTab, changes)}
            onConnect={() => handleConnectClient(activeTab)}
            onDisconnect={() => handleDisconnect(activeTab)}
          />
        ) : (
          <div className="empty-state">
            <p>No tabs open. Click '+ New Tab' to create one.</p>
          </div>
        )}
      </div>

      <StatusBar tab={activeTab < tabs.length ? tabs[activeTab] : null} />

      {showNewTabDialog && (
        <div className="dialog-overlay">
          <div className="dialog">
            <h2>New Connection</h2>
            <p>Select connection type:</p>
            <div className="dialog-buttons">
              <button onClick={() => handleAddTab("Client")}>
                📡 Client Connection
              </button>
              <button onClick={() => handleAddTab("Server")}>
                🏠 Server (Listen)
              </button>
            </div>
            <button
              className="cancel-btn"
              onClick={() => setShowNewTabDialog(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {showSettings && <SettingsWindow onClose={() => toggleSettings()} />}
    </div>
  );
}

export default App;
