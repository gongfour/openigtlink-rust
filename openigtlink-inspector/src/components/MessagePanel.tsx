import { Tab } from "../App";
import MessageList from "./MessageList";
import SendPanel from "./SendPanel";
import { useAppStore } from "../store/appStore";
import "./MessagePanel.css";

interface MessagePanelProps {
  tab: Tab;
  tabIndex: number;
  onTabChange: (changes: Partial<Tab>) => void;
  onConnect: () => void;
  onDisconnect: () => void;
}

export default function MessagePanel({
  tab,
  tabIndex,
  onTabChange,
  onConnect,
  onDisconnect,
}: MessagePanelProps) {
  // Get raw messages and filters separately with stable selectors
  const tabMessages = useAppStore((state) => state.tabMessages.get(tab.id) || []);
  const searchText = useAppStore((state) => state.filters.searchText);
  const selectedTypes = useAppStore((state) => state.filters.selectedTypes);
  const selectedDevices = useAppStore((state) => state.filters.selectedDevices);

  const {
    setSearchText,
    setSelectedTypes,
    setSelectedDevices,
    clearFilters,
  } = useAppStore();

  // Apply filtering locally
  const messages = tabMessages.filter((msg) => {
    // Search text filter
    if (
      searchText &&
      !msg.device_name.toLowerCase().includes(searchText.toLowerCase()) &&
      !msg.message_type.toLowerCase().includes(searchText.toLowerCase())
    ) {
      return false;
    }

    // Message type filter
    if (
      selectedTypes.length > 0 &&
      !selectedTypes.includes(msg.message_type)
    ) {
      return false;
    }

    // Device name filter
    if (
      selectedDevices.length > 0 &&
      !selectedDevices.includes(msg.device_name)
    ) {
      return false;
    }

    return true;
  });

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

      <div className="search-widget">
        <div className="search-controls">
          <label>Search:</label>
          <input
            type="text"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder="Search by device name or message type..."
            className="search-input"
          />
          {(searchText ||
            selectedTypes.length > 0 ||
            selectedDevices.length > 0) && (
            <button onClick={() => clearFilters()} className="clear-filters-btn">
              Clear Filters
            </button>
          )}
        </div>
        <div className="search-info">
          {messages.length > 0 ? (
            <span>
              Showing {messages.length} message(s)
              {searchText && ` matching "${searchText}"`}
            </span>
          ) : (
            <span>No messages match the current filters</span>
          )}
        </div>
      </div>

      <div className="separator"></div>

      <div className="messages-area">
        <MessageList messages={messages} tabIndex={tabIndex} />
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
