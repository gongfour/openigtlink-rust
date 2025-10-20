# OpenIGTLink Inspector - Tauri Edition

A modern desktop application for inspecting, testing, and debugging OpenIGTLink protocol messages. Built with **Tauri**, **React**, and **TypeScript** for cross-platform performance.

## Project Structure

```
openigtlink-inspector/
├── src/                          # React Frontend (TypeScript)
│   ├── components/              # React UI components
│   │   ├── TopBar.tsx          # Header with title and settings
│   │   ├── TabBar.tsx          # Tab management
│   │   ├── MessagePanel.tsx    # Main message display area
│   │   ├── MessageList.tsx     # Message table
│   │   ├── SendPanel.tsx       # Message sender UI
│   │   ├── StatusBar.tsx       # Status display
│   │   └── SettingsWindow.tsx  # Settings dialog
│   ├── App.tsx                  # Main React app component
│   ├── main.tsx                 # React entry point
│   └── index.css               # Global styles
├── src-tauri/                    # Rust Backend (Tauri)
│   ├── src/
│   │   ├── main.rs             # Tauri app entry point
│   │   ├── commands.rs         # Tauri IPC commands
│   │   ├── connection.rs       # OpenIGTLink connection logic
│   │   └── types.rs            # Shared data types
│   ├── build.rs                # Tauri build script
│   └── Cargo.toml              # Rust dependencies
├── package.json                # Node.js dependencies
├── vite.config.ts              # Vite bundler config
├── tsconfig.json               # TypeScript config
├── index.html                  # HTML entry point
└── README.md                   # This file
```

## Tech Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Fast bundler
- **CSS** - Styling

### Backend
- **Rust** - Performance and safety
- **Tauri 1.5** - Desktop app framework
- **Tokio** - Async runtime
- **OpenIGTLink-Rust** - Protocol implementation

## Features

### Current (MVP)
- ✅ Multiple connection tabs (Client/Server mode)
- ✅ Real-time message reception and display
- ✅ Message type color-coding
- ✅ Connection status monitoring
- ✅ Settings panel
- ✅ Clean, intuitive UI

### In Development
- 📝 Message sending functionality
- 📝 Message filtering and search
- 📝 Server mode implementation
- 📝 Message type-specific editors

### Future Enhancements
- 🔮 Image visualization
- 🔮 3D transform visualization
- 🔮 Real-time sensor plotting
- 🔮 Message recording/replay
- 🔮 Message validation

## Getting Started

### Prerequisites
- Node.js 18+
- Rust 1.70+
- npm or yarn

### Installation

```bash
cd openigtlink-inspector

# Install Node dependencies
npm install

# Build React frontend
npm run build
```

### Development

Start the development server (requires separate Tauri dev setup):

```bash
npm run dev           # Start Vite dev server
npm run tauri-dev     # Run Tauri in dev mode (separate terminal)
```

### Build for Production

```bash
npm run tauri-build
```

Output binaries will be in `src-tauri/target/release/bundle/`

## Architecture

### Communication Flow

```
┌─────────────────────────────────────────┐
│         React Frontend (TypeScript)     │
│  - UI Components                        │
│  - State Management                     │
│  - Event Listeners                      │
└─────────────────────────────────────────┘
              ↓ Tauri IPC ↑
       (Commands & Events)
┌─────────────────────────────────────────┐
│      Tauri Backend (Rust)               │
│  - Connection Management                │
│  - Message Processing                   │
│  - OpenIGTLink Protocol                 │
│  - Tokio Async Runtime                  │
└─────────────────────────────────────────┘
              ↓ TCP Socket ↑
┌─────────────────────────────────────────┐
│   OpenIGTLink Server/Client             │
│   (3D Slicer, PLUS Toolkit, etc.)       │
└─────────────────────────────────────────┘
```

### Key Components

**Frontend:**
- `App.tsx` - Main state container and tab management
- `MessagePanel.tsx` - Connection controls and message display
- `Tauri API integration` - `@tauri-apps/api` for IPC

**Backend:**
- `main.rs` - Tauri app setup and command handlers
- `connection.rs` - Manages async OpenIGTLink connections
- `commands.rs` - Exported Tauri commands for frontend

## Comparison with Previous egui Version

| Aspect | egui (Previous) | Tauri (Current) |
|--------|-----------------|-----------------|
| **Framework** | egui (Rust) | Tauri + React |
| **UI Framework** | Immediate Mode GUI | React Components |
| **State Management** | Rust structs | React hooks |
| **Styling** | Egui theme system | CSS |
| **Bundle Size** | Small | Medium |
| **Development Speed** | Slower | Faster |
| **UI Customization** | Limited | Extensive |

## Configuration

### Tauri Settings (`src-tauri/tauri.conf.json`)
- Window size: 1400x900 (minimum 1000x700)
- App name: "openigtlink-inspector"
- CSP: Disabled for development

### Backend (`src-tauri/Cargo.toml`)
- Tauri 1.5
- Tokio for async runtime
- OpenIGTLink-Rust library

## Troubleshooting

### Build Issues

**"workspace member not found"**
- The frontend is a Node.js project, not a Cargo package
- Backend is in `src-tauri/` and builds independently

**Compilation errors in Rust**
```bash
cd src-tauri
cargo check
cargo build
```

**Node dependency issues**
```bash
rm -rf node_modules package-lock.json
npm install
```

## Development Tips

1. **Hot Reload**: Vite provides fast HMR during development
2. **Tauri Dev**: Use `npm run tauri-dev` for backend changes
3. **Debugging**: Use browser DevTools in Tauri window
4. **Logging**: Check console output and Rust logs via `tracing`

## Performance Considerations

- Message buffer limited to 1000 entries (configurable in settings)
- UI updates at 60 FPS (Tauri default)
- Async message receiving prevents UI blocking
- Efficient table rendering with React virtualization (future)

## Cross-Platform Support

- ✅ macOS (Intel & Apple Silicon)
- ✅ Windows (x64)
- ✅ Linux (x64, Wayland/X11)

## License

MIT - Same as OpenIGTLink-Rust

## Related Files

- Backup of egui version: `../openigtlink-inspector.egui-backup/`
- OpenIGTLink library: `../../openigtlink-rust/`
- Main workspace: `../../`
