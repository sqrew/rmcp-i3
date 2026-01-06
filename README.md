# rmcp-i3

MCP server for controlling the [i3 window manager](https://i3wm.org/) via IPC.

Built with [rmcp](https://crates.io/crates/rmcp) (Rust Model Context Protocol).

## Features

- **get_workspaces** - List all workspaces with their properties
- **get_tree** - Get the full window tree (containers, windows, layout)
- **switch_workspace** - Switch to a workspace by number or name
- **focus_window** - Focus a window by i3 criteria (class, title, etc.)
- **move_to_workspace** - Move the focused window to a workspace
- **exec** - Launch an application
- **kill** - Close the focused window
- **kill_window** - Close a window by criteria (safer than kill)
- **fullscreen** - Toggle fullscreen mode
- **run_command** - Execute any i3 command (escape hatch)

## Installation

```bash
# From source
git clone https://github.com/sqrew/rmcp-i3
cd rmcp-i3
cargo install --path .

# Or build and run directly
cargo build --release
./target/release/rmcp-i3
```

## Usage

### Claude Code (feel free to use your own, of course)

Add to your `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "i3": {
      "command": "/path/to/rmcp-i3"
    }
  }
}
```

Then Claude can control your i3 setup:

```
> Switch me to workspace 3
> Focus the Firefox window
> Move this window to workspace "code"
> Show me all my workspaces
```

### Programmatic

The server speaks MCP over stdio. Send JSON-RPC 2.0 messages to interact with it.

## Tool Reference

### get_workspaces

Lists all workspaces with: number, name, visible, focused, urgent, output.

### get_tree

Returns the full i3 container tree as JSON. Useful for understanding window layout.

### switch_workspace

**Parameters:**
- `workspace` (string) - Workspace to switch to. Can be a number ("1") or name ("web").

### focus_window

**Parameters:**
- `criteria` (string) - i3 criteria to match. Examples:
  - `[class="Firefox"]` - Match by window class
  - `[title="vim"]` - Match by title
  - `[instance="spotify"]` - Match by instance
  - `[class="Alacritty" title="nvim"]` - Multiple criteria

### move_to_workspace

**Parameters:**
- `workspace` (string) - Destination workspace for the focused window.

### exec

**Parameters:**
- `command` (string) - Application to launch. Examples:
  - `firefox` - Open Firefox
  - `kitty` - Open a new terminal
  - `emacs` - Open Emacs

### kill

Closes the currently focused window. No parameters.

### kill_window

**Parameters:**
- `criteria` (string) - i3 criteria to match window to kill. Examples:
  - `[class="Firefox"]` - Kill Firefox
  - `[title="~"]` - Kill window with title "~"
  - `[class="kitty" title="htop"]` - Kill kitty running htop

### fullscreen

Toggles fullscreen mode for the currently focused window. No parameters.

### run_command

**Parameters:**
- `command` (string) - Any valid i3 command. Examples:
  - `split h` / `split v` - Split container
  - `layout tabbed` / `layout stacking` - Change layout
  - `kill` - Close focused window
  - `fullscreen toggle` - Toggle fullscreen
  - `floating toggle` - Toggle floating mode

See the [i3 user guide](https://i3wm.org/docs/userguide.html#list_of_commands) for full command list.

## Requirements

- i3 window manager running
- Rust 1.70+ (for building)

## License

MIT

## Credits

Built with:
- [rmcp](https://crates.io/crates/rmcp) - Rust MCP SDK
- [tokio-i3ipc](https://crates.io/crates/tokio-i3ipc) - Async i3 IPC bindings
