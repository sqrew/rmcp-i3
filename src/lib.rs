//! rmcp-i3: MCP server for i3 window manager control
//!
//! Provides tools to query and control i3 via IPC.

use rmcp::{
    handler::server::{router::tool::ToolRouter, ServerHandler, wrapper::Parameters},
    model::*,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio_i3ipc::{
    reply::{Node, Workspace},
    I3,
};
use tracing::{debug, error, info};

// ============================================================================
// Server Struct
// ============================================================================

/// MCP server for i3 window manager control
#[derive(Debug)]
pub struct I3Server {
    /// Tool router for MCP tool dispatch
    pub tool_router: ToolRouter<Self>,
}

impl I3Server {
    /// Create a new i3 MCP server
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Connect to i3 IPC socket
    async fn connect(&self) -> Result<I3, McpError> {
        I3::connect()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to connect to i3: {}", e), None))
    }
}

impl Default for I3Server {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tool Parameters
// ============================================================================

/// Parameters for switch_workspace tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SwitchWorkspaceParams {
    /// Workspace to switch to (number or name, e.g. "1", "web", "music")
    #[schemars(description = "Workspace to switch to (number or name)")]
    pub workspace: String,
}

/// Parameters for focus_window tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FocusWindowParams {
    /// i3 criteria to match window (e.g. "[class=\"Firefox\"]", "[title=\"vim\"]")
    #[schemars(description = "i3 criteria to match window, e.g. [class=\"Firefox\"] or [title=\"vim\"]")]
    pub criteria: String,
}

/// Parameters for move_to_workspace tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MoveToWorkspaceParams {
    /// Workspace to move the focused window to
    #[schemars(description = "Workspace to move the focused window to")]
    pub workspace: String,
}

/// Parameters for run_command tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunCommandParams {
    /// i3 command to execute (see i3 user guide for full command list)
    #[schemars(description = "i3 command to execute (e.g. 'split h', 'layout tabbed', 'kill')")]
    pub command: String,
}

/// Parameters for exec tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecParams {
    /// Command to execute (application to launch)
    #[schemars(description = "Command to execute, e.g. 'firefox', 'kitty', 'emacs'")]
    pub command: String,
}

/// Parameters for kill_window tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KillWindowParams {
    /// i3 criteria to match window to kill (e.g. "[class=\"Firefox\"]", "[title=\"vim\"]")
    #[schemars(description = "i3 criteria to match window to kill, e.g. [class=\"Firefox\"] or [title=\"~\"]")]
    pub criteria: String,
}

// ============================================================================
// Tool Implementations
// ============================================================================

#[rmcp::tool_router]
impl I3Server {
    /// Get a list of all workspaces with their properties
    #[rmcp::tool(description = "List all i3 workspaces with their properties (number, name, visible, focused, urgent, output)")]
    pub async fn get_workspaces(&self) -> Result<CallToolResult, McpError> {
        info!("Getting workspaces");
        let mut conn = self.connect().await?;

        let workspaces: Vec<Workspace> = conn.get_workspaces().await.map_err(|e| {
            error!("Failed to get workspaces: {}", e);
            McpError::internal_error(format!("Failed to get workspaces: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&workspaces).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize workspaces: {}", e), None)
        })?;

        debug!("Found {} workspaces", workspaces.len());
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get the full i3 window tree
    #[rmcp::tool(description = "Get the full i3 window tree (all containers, windows, and their layout)")]
    pub async fn get_tree(&self) -> Result<CallToolResult, McpError> {
        info!("Getting window tree");
        let mut conn = self.connect().await?;

        let tree: Node = conn.get_tree().await.map_err(|e| {
            error!("Failed to get tree: {}", e);
            McpError::internal_error(format!("Failed to get tree: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&tree).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize tree: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Switch to a specific workspace
    #[rmcp::tool(description = "Switch to a specific workspace by number or name")]
    pub async fn switch_workspace(
        &self,
        Parameters(params): Parameters<SwitchWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Switching to workspace: {}", params.workspace);
        let mut conn = self.connect().await?;

        let command = format!("workspace {}", params.workspace);
        let results = conn.run_command(&command).await.map_err(|e| {
            error!("Failed to switch workspace: {}", e);
            McpError::internal_error(format!("Failed to switch workspace: {}", e), None)
        })?;

        // Check if command succeeded
        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Switched to workspace '{}'",
                params.workspace
            ))]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to switch workspace: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Focus a window by i3 criteria
    #[rmcp::tool(description = "Focus a window matching i3 criteria (e.g. [class=\"Firefox\"], [title=\"vim\"])")]
    pub async fn focus_window(
        &self,
        Parameters(params): Parameters<FocusWindowParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Focusing window: {}", params.criteria);
        let mut conn = self.connect().await?;

        let command = format!("{} focus", params.criteria);
        let results = conn.run_command(&command).await.map_err(|e| {
            error!("Failed to focus window: {}", e);
            McpError::internal_error(format!("Failed to focus window: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Focused window matching '{}'",
                params.criteria
            ))]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to focus window: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Move the focused window to a workspace
    #[rmcp::tool(description = "Move the currently focused window to a specific workspace")]
    pub async fn move_to_workspace(
        &self,
        Parameters(params): Parameters<MoveToWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Moving window to workspace: {}", params.workspace);
        let mut conn = self.connect().await?;

        let command = format!("move container to workspace {}", params.workspace);
        let results = conn.run_command(&command).await.map_err(|e| {
            error!("Failed to move window: {}", e);
            McpError::internal_error(format!("Failed to move window: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Moved window to workspace '{}'",
                params.workspace
            ))]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to move window: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Run an arbitrary i3 command
    #[rmcp::tool(description = "Execute any i3 command (escape hatch for advanced operations). See i3 user guide for command list.")]
    pub async fn run_command(
        &self,
        Parameters(params): Parameters<RunCommandParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Running i3 command: {}", params.command);
        let mut conn = self.connect().await?;

        let results = conn.run_command(&params.command).await.map_err(|e| {
            error!("Failed to run command: {}", e);
            McpError::internal_error(format!("Failed to run command: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&results).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize results: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Launch an application
    #[rmcp::tool(description = "Launch an application (e.g. 'firefox', 'kitty', 'emacs')")]
    pub async fn exec(
        &self,
        Parameters(params): Parameters<ExecParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Executing: {}", params.command);
        let mut conn = self.connect().await?;

        let command = format!("exec {}", params.command);
        let results = conn.run_command(&command).await.map_err(|e| {
            error!("Failed to exec: {}", e);
            McpError::internal_error(format!("Failed to exec: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Launched '{}'",
                params.command
            ))]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to launch: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Kill (close) the focused window
    #[rmcp::tool(description = "Kill (close) the currently focused window")]
    pub async fn kill(&self) -> Result<CallToolResult, McpError> {
        info!("Killing focused window");
        let mut conn = self.connect().await?;

        let results = conn.run_command("kill").await.map_err(|e| {
            error!("Failed to kill window: {}", e);
            McpError::internal_error(format!("Failed to kill window: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(
                "Killed focused window".to_string(),
            )]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to kill window: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Kill (close) a window matching criteria
    #[rmcp::tool(description = "Kill (close) a window matching i3 criteria (e.g. [class=\"Firefox\"], [title=\"vim\"])")]
    pub async fn kill_window(
        &self,
        Parameters(params): Parameters<KillWindowParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Killing window: {}", params.criteria);
        let mut conn = self.connect().await?;

        let command = format!("{} kill", params.criteria);
        let results = conn.run_command(&command).await.map_err(|e| {
            error!("Failed to kill window: {}", e);
            McpError::internal_error(format!("Failed to kill window: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Killed window matching '{}'",
                params.criteria
            ))]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to kill window: {}",
                errors.join(", ")
            ))]))
        }
    }

    /// Toggle fullscreen for the focused window
    #[rmcp::tool(description = "Toggle fullscreen mode for the currently focused window")]
    pub async fn fullscreen(&self) -> Result<CallToolResult, McpError> {
        info!("Toggling fullscreen");
        let mut conn = self.connect().await?;

        let results = conn.run_command("fullscreen toggle").await.map_err(|e| {
            error!("Failed to toggle fullscreen: {}", e);
            McpError::internal_error(format!("Failed to toggle fullscreen: {}", e), None)
        })?;

        let success = results.iter().all(|r| r.success);
        if success {
            Ok(CallToolResult::success(vec![Content::text(
                "Toggled fullscreen".to_string(),
            )]))
        } else {
            let errors: Vec<String> = results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to toggle fullscreen: {}",
                errors.join(", ")
            ))]))
        }
    }
}

// ============================================================================
// Server Handler Implementation
// ============================================================================

#[rmcp::tool_handler]
impl ServerHandler for I3Server {
    fn get_info(&self) -> ServerInfo {
        InitializeResult {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: None }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "rmcp-i3".to_string(),
                title: Some("i3 Window Manager MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "MCP server for controlling the i3 window manager. \
                 Use get_workspaces to list workspaces, get_tree for window layout, \
                 switch_workspace/focus_window/move_to_workspace for navigation, \
                 and run_command for arbitrary i3 commands."
                    .to_string(),
            ),
        }
    }
}
