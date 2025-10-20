import { create } from "zustand";
import { Tab, ReceivedMessage } from "../App";

interface Settings {
  theme: "dark" | "light" | "auto";
  bufferSize: number;
  autoScroll: boolean;
  autoReconnect: boolean;
  verifyCrc: boolean;
}

interface SendPanelState {
  messageType: string;
  deviceName: string;
  toClient: string;
  content: string;
  isRepeating: boolean;
  repeatRate: number;
}

interface AppState {
  // State
  tabs: Tab[];
  activeTab: number;
  nextTabId: number;
  tabMessages: Map<number, ReceivedMessage[]>;
  showNewTabDialog: boolean;
  showSettings: boolean;
  settings: Settings;
  sendPanel: SendPanelState;
  expandedMessageKeys: Set<string>;

  // Tab actions
  setTabs: (tabs: Tab[]) => void;
  setActiveTab: (index: number) => void;
  setNextTabId: (id: number) => void;
  addTab: (tab: Tab) => void;
  removeTab: (index: number) => void;
  updateTab: (index: number, changes: Partial<Tab>) => void;

  // Message actions
  addTabMessage: (tabId: number, message: ReceivedMessage) => void;
  setTabMessages: (tabId: number, messages: ReceivedMessage[]) => void;
  clearTabMessages: (tabId: number) => void;
  getTabMessages: (tabId: number) => ReceivedMessage[];

  // UI actions
  setShowNewTabDialog: (show: boolean) => void;
  setShowSettings: (show: boolean) => void;
  toggleSettings: () => void;

  // Connection actions
  setTabConnected: (index: number, connected: boolean) => void;
  setTabError: (index: number, error: string | undefined) => void;
  incrementRxCount: (index: number) => void;
  incrementTxCount: (index: number) => void;

  // Settings actions
  updateSettings: (settings: Partial<Settings>) => void;
  setTheme: (theme: "dark" | "light" | "auto") => void;
  setBufferSize: (size: number) => void;
  setAutoScroll: (autoScroll: boolean) => void;
  setAutoReconnect: (autoReconnect: boolean) => void;
  setVerifyCrc: (verifyCrc: boolean) => void;

  // SendPanel actions
  updateSendPanel: (data: Partial<SendPanelState>) => void;
  clearSendPanel: () => void;

  // MessageList actions
  toggleMessageExpanded: (messageKey: string) => void;
  clearExpandedMessages: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  tabs: [
    {
      id: 0,
      name: "Client-0",
      tab_type: "Client",
      host: "127.0.0.1",
      port: "18944",
      is_connected: false,
      send_panel_expanded: false,
      rx_count: 0,
      tx_count: 0,
    },
  ],
  activeTab: 0,
  nextTabId: 1,
  tabMessages: new Map<number, ReceivedMessage[]>([[0, []]]),
  showNewTabDialog: false,
  showSettings: false,
  settings: {
    theme: "dark",
    bufferSize: 1000,
    autoScroll: true,
    autoReconnect: false,
    verifyCrc: true,
  },
  sendPanel: {
    messageType: "TRANSFORM",
    deviceName: "TestDevice",
    toClient: "All Clients",
    content: "",
    isRepeating: false,
    repeatRate: 60,
  },
  expandedMessageKeys: new Set<string>(),

  // Tab actions
  setTabs: (tabs) => set({ tabs }),
  setActiveTab: (index) => set({ activeTab: index }),
  setNextTabId: (id) => set({ nextTabId: id }),

  addTab: (tab) =>
    set((state) => {
      const newMap = new Map(state.tabMessages);
      newMap.set(tab.id, []);
      return {
        tabs: [...state.tabs, tab],
        activeTab: state.tabs.length,
        tabMessages: newMap,
      };
    }),

  removeTab: (index) =>
    set((state) => {
      const tabToRemove = state.tabs[index];
      const updated = state.tabs.filter((_, i) => i !== index);
      let newActiveTab = state.activeTab;
      if (newActiveTab >= updated.length && newActiveTab > 0) {
        newActiveTab = newActiveTab - 1;
      }

      // Remove messages for this tab
      const newMap = new Map(state.tabMessages);
      newMap.delete(tabToRemove.id);

      return {
        tabs: updated,
        activeTab: newActiveTab,
        tabMessages: newMap,
      };
    }),

  updateTab: (index, changes) =>
    set((state) => {
      const updated = [...state.tabs];
      updated[index] = { ...updated[index], ...changes };
      return { tabs: updated };
    }),

  // Message actions
  addTabMessage: (tabId, message) =>
    set((state) => {
      const messages = state.tabMessages.get(tabId) || [];
      const updated = [message, ...messages].slice(0, 1000);
      const newMap = new Map(state.tabMessages);
      newMap.set(tabId, updated);
      return { tabMessages: newMap };
    }),

  setTabMessages: (tabId, messages) =>
    set((state) => {
      const newMap = new Map(state.tabMessages);
      newMap.set(tabId, messages);
      return { tabMessages: newMap };
    }),

  clearTabMessages: (tabId) =>
    set((state) => {
      const newMap = new Map(state.tabMessages);
      newMap.set(tabId, []);
      return { tabMessages: newMap };
    }),

  getTabMessages: (tabId) => {
    return get().tabMessages.get(tabId) || [];
  },

  // UI actions
  setShowNewTabDialog: (show) => set({ showNewTabDialog: show }),
  setShowSettings: (show) => set({ showSettings: show }),
  toggleSettings: () => set((state) => ({ showSettings: !state.showSettings })),

  // Connection actions
  setTabConnected: (index, connected) =>
    set((state) => {
      const updated = [...state.tabs];
      if (updated[index]) {
        updated[index].is_connected = connected;
      }
      return { tabs: updated };
    }),

  setTabError: (index, error) =>
    set((state) => {
      const updated = [...state.tabs];
      if (updated[index]) {
        updated[index].error_message = error;
      }
      return { tabs: updated };
    }),

  incrementRxCount: (index) =>
    set((state) => {
      const updated = [...state.tabs];
      if (updated[index]) {
        updated[index].rx_count += 1;
      }
      return { tabs: updated };
    }),

  incrementTxCount: (index) =>
    set((state) => {
      const updated = [...state.tabs];
      if (updated[index]) {
        updated[index].tx_count += 1;
      }
      return { tabs: updated };
    }),

  // Settings actions
  updateSettings: (newSettings) =>
    set((state) => ({
      settings: { ...state.settings, ...newSettings },
    })),

  setTheme: (theme) =>
    set((state) => ({
      settings: { ...state.settings, theme },
    })),

  setBufferSize: (bufferSize) =>
    set((state) => ({
      settings: { ...state.settings, bufferSize },
    })),

  setAutoScroll: (autoScroll) =>
    set((state) => ({
      settings: { ...state.settings, autoScroll },
    })),

  setAutoReconnect: (autoReconnect) =>
    set((state) => ({
      settings: { ...state.settings, autoReconnect },
    })),

  setVerifyCrc: (verifyCrc) =>
    set((state) => ({
      settings: { ...state.settings, verifyCrc },
    })),

  // SendPanel actions
  updateSendPanel: (data) =>
    set((state) => ({
      sendPanel: { ...state.sendPanel, ...data },
    })),

  clearSendPanel: () =>
    set({
      sendPanel: {
        messageType: "TRANSFORM",
        deviceName: "TestDevice",
        toClient: "All Clients",
        content: "",
        isRepeating: false,
        repeatRate: 60,
      },
    }),

  // MessageList actions
  toggleMessageExpanded: (messageKey) =>
    set((state) => {
      const newExpanded = new Set(state.expandedMessageKeys);
      if (newExpanded.has(messageKey)) {
        newExpanded.delete(messageKey);
      } else {
        newExpanded.add(messageKey);
      }
      return { expandedMessageKeys: newExpanded };
    }),

  clearExpandedMessages: () => set({ expandedMessageKeys: new Set<string>() }),
}));
