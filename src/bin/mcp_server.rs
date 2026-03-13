// src/bin/mcp_server.rs - MCP HTTP Server (Port 4040)
// Entry point: cargo run --bin mcp_server
// Or: cargo run --bin mcp_server -- --port 4040

use clap::Parser;
use memory_p::mcp::init_http_mcp;

#[derive(Parser, Debug)]
#[command(name = "MEMORY_P MCP Server")]
#[command(about = "HTTP MCP Server for MEMORY_P v3.0", long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "4040")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("╔════════════════════════════════════════════════════╗");
    println!("║     MEMORY_P v3.0 - MCP HTTP Server                ║");
    println!("║     Model Context Protocol over HTTP/REST          ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!();
    println!("Starting server on port {}...", args.port);
    println!();

    init_http_mcp(args.port).await?;

    Ok(())
}
