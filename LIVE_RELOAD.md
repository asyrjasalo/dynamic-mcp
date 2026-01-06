# Live Reload Feature

## Overview

The modular-mcp server now supports live reloading of downstream resources when the configuration file is modified. This feature automatically detects changes to the config file and reconnects to all MCP servers with the updated configuration.

## How It Works

1. **File Watching**: Uses the `notify` crate to monitor the configuration file for changes
2. **Automatic Reload**: When changes are detected, the server:
   - Disconnects from all existing upstream MCP servers
   - Reloads the configuration file
   - Reconnects to all servers with the new configuration
3. **Error Handling**: Failed connections are logged but don't stop the reload process
4. **Live Updates**: The MCP proxy continues serving requests during the reload

## Implementation Details

### Components Added

1. **`src/watcher.rs`**: File watcher module using `notify` crate
2. **Modified `src/main.rs`**: Added reload logic with async channel
3. **Modified `src/server.rs`**: Changed to use `Arc<RwLock<>>` for concurrent access
4. **Modified `Cargo.toml`**: Added `notify = "6.1"` dependency

### Key Features

- **Non-blocking**: Reload happens in background without interrupting the server
- **Concurrent access**: Uses `RwLock` to allow reads during reload
- **Comprehensive logging**: All reload events are logged with appropriate levels
- **Error resilience**: Failed reconnections are logged but don't crash the server

## Usage

Simply edit your configuration file while the server is running:

```bash
# Start the server
./target/release/modular-mcp config.json

# In another terminal, edit the config
vim config.json

# The server automatically detects the change and reloads
```

## Log Messages

When a reload occurs, you'll see messages like:

```
INFO Config file changed, triggering reload
INFO Config file changed, reloading...
INFO Successfully disconnected from all groups
INFO ✅ Successfully reconnected to MCP group: filesystem
INFO ✅ Config reload complete: 2 groups connected
```

## Testing

To manually test the live reload feature:

1. Build the project:
   ```bash
   cargo build --release
   ```

2. Start the server with a test config:
   ```bash
   ./target/release/modular-mcp config.example.json
   ```

3. In another terminal, modify the config file:
   ```bash
   # Add a new server or change existing ones
   vim config.example.json
   ```

4. Watch the first terminal for reload messages

## Technical Notes

- The file watcher is kept alive using `std::mem::forget()` to prevent it from being dropped
- Configuration reload happens in a spawned task to avoid blocking the main server
- All existing connections are properly closed before reconnecting
- The reload is atomic from the client's perspective (uses RwLock)

## Error Handling

If the configuration file has errors during reload:
- The error is logged
- Previous connections remain disconnected
- The server continues running (doesn't crash)
- Fix the config and save again to trigger another reload attempt
