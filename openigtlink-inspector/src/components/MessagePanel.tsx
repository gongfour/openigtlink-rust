import { Tab } from "../App";
import MessageList from "./MessageList";
import SendPanel from "./SendPanel";
import { useAppStore } from "../store/appStore";
import "./MessagePanel.css";

interface MessagePanelProps {
  tab: Tab;
  onTabChange: (changes: Partial<Tab>) => void;
  onConnect: () => void;
  onDisconnect: () => void;
}

export default function MessagePanel({
  tab,
  onTabChange,
  onConnect,
  onDisconnect,
}: MessagePanelProps) {
  // Get filtered messages for this specific tab
  const messages = useAppStore((state) => state.getFilteredMessages(tab.id));
  const {
    filters,
    setSearchText,
    setSelectedTypes,
    setSelectedDevices,
    clearFilters,
  } = useAppStore();

  const handleHostChange = (host: string) => {
    onTabChange({ host });
  };

  const handlePortChange = (port: string) => {
    onTabChange({ port });
  };

  const handleToggleSendPanel = () => {
    onTabChange({ send_panel_expanded: !tab.send_panel_expanded });
  };

  return (
    <div className="message-panel">
      <div className="connection-controls">
        <div className="control-group">
          <label>Host:</label>
          <input
            type="text"
            value={tab.host}
            onChange={(e) => handleHostChange(e.target.value)}
            disabled={tab.is_connected}
          />
        </div>

        <div className="control-group">
          <label>Port:</label>
          <input
            type="text"
            value={tab.port}
            onChange={(e) => handlePortChange(e.target.value)}
            disabled={tab.is_connected}
            style={{ width: "80px" }}
          />
        </div>

        <button
          className="connect-btn"
          onClick={tab.is_connected ? onDisconnect : onConnect}
        >
          {tab.is_connected ? "Disconnect" : "Connect"}
        </button>

        {tab.is_connected ? (
          <span className="status-connected">● Connected</span>
        ) : (
          <span className="status-disconnected">○ Disconnected</span>
        )}

        {tab.error_message && (
          <span className="error-message">{tab.error_message}</span>
        )}
      </div>

      <div className="separator"></div>

      <div
        style={{
          padding: "10px",
          backgroundColor: "#f5f5f5",
          borderRadius: "4px",
        }}
      >
        <div style={{ marginBottom: "10px" }}>
          <label style={{ fontSize: "12px", fontWeight: "bold" }}>
            Search:{" "}
          </label>
          <input
            type="text"
            value={filters.searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder="Search by device name or message type..."
            style={{
              padding: "4px 8px",
              marginLeft: "8px",
              width: "300px",
              borderRadius: "4px",
              border: "1px solid #ccc",
            }}
          />
          {(filters.searchText ||
            filters.selectedTypes.length > 0 ||
            filters.selectedDevices.length > 0) && (
            <button
              onClick={() => clearFilters()}
              style={{
                marginLeft: "10px",
                padding: "4px 12px",
                backgroundColor: "#ff9800",
                color: "white",
                border: "none",
                borderRadius: "4px",
                cursor: "pointer",
                fontSize: "12px",
              }}
            >
              Clear Filters
            </button>
          )}
        </div>
        <div style={{ fontSize: "12px", color: "#666" }}>
          {messages.length > 0 ? (
            <span>
              Showing {messages.length} message(s)
              {filters.searchText && ` matching "${filters.searchText}"`}
            </span>
          ) : (
            <span>No messages match the current filters</span>
          )}
        </div>
      </div>

      <div className="separator"></div>

      <div className="messages-area">
        <MessageList messages={messages} />
      </div>

      <div className="separator"></div>

      <SendPanel
        isExpanded={tab.send_panel_expanded}
        onToggle={handleToggleSendPanel}
        showToSelector={tab.tab_type === "Server"}
        tabId={tab.id}
      />
    </div>
  );
}
