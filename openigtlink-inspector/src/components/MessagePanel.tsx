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
  // Get messages for this specific tab
  const messages = useAppStore((state) => state.getTabMessages(tab.id));

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
