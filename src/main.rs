//! rmcp-i3: MCP server for i3 window manager control
//!
//! Run with: `rmcp-i3` (serves on stdio)

use rmcp::ServiceExt;
use rmcp_i3::I3Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (to stderr so it doesn't interfere with stdio transport)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting rmcp-i3 server");

    // Create server and serve on stdio
    let server = I3Server::new();
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for shutdown
    service.waiting().await?;

    tracing::info!("rmcp-i3 server stopped");
    Ok(())
}
