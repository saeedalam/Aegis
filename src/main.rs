//! Aegis - MCP Tool Server for AI Agents
//!
//! A high-performance, single-binary MCP server providing tools
//! for AI agents using the Model Context Protocol (MCP).
//!
//! ## Usage
//!
//! ```bash
//! # Execute a single tool and exit
//! aegis run echo --args '{"text": "Hello!"}'
//!
//! # Start in stdio mode (for CLI/pipe-based clients)
//! aegis --stdio
//!
//! # Start HTTP/SSE server
//! aegis serve --port 9000
//! ```

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter};

use aegis::core::{Config, RuntimeState};
use aegis::handlers::Router;
use aegis::transport::{Transport, StdioTransport};
use aegis::transport::sse::{SseState, start_server};

/// Aegis - MCP Tool Server for AI Agents
#[derive(Parser, Debug)]
#[command(name = "aegis")]
#[command(author, version, about = "MCP Tool Server for AI Agents")]
#[command(after_help = "EXAMPLES:\n  \
    aegis run echo --args '{\"text\": \"hello\"}'\n  \
    aegis serve --port 9000\n  \
    aegis --stdio")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "aegis.json")]
    config: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run in stdio mode (JSON-RPC over stdin/stdout)
    #[arg(long)]
    stdio: bool,

    /// Disable extra tools (load only core tools)
    #[arg(long)]
    core_only: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the HTTP/SSE server
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value_t = 9000)]
        port: u16,
    },

    /// Execute a single tool and exit
    Run {
        /// Tool name to execute
        tool: String,

        /// JSON arguments for the tool
        #[arg(short, long, default_value = "{}")]
        args: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// List all available tools
    Tools,

    /// Show server version and capabilities
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging (to stderr to avoid corrupting JSON-RPC on stdout)
    init_logging(&cli.log_level);

    // Load configuration
    let mut config = Config::load_from_file(&cli.config).unwrap_or_else(|e| {
        info!("Could not load config file: {}. Using defaults.", e);
        Config::default()
    });

    // Handle --core-only flag
    if cli.core_only {
        config.extras_enabled = false;
    }

    // Handle stdio mode
    if cli.stdio {
        return run_stdio_mode(config).await;
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Serve { host, port }) => {
            config.host = host;
            config.port = port;
            run_serve_mode(config).await
        }
        Some(Commands::Run { tool, args, format }) => {
            run_oneshot_mode(config, &tool, &args, &format).await
        }
        Some(Commands::Tools) => {
            list_tools(config).await
        }
        Some(Commands::Info) => {
            show_info(&config);
            Ok(())
        }
        None => {
            // Default: show banner and usage
            print_banner(&config);
            print_quick_start();
            Ok(())
        }
    }
}

/// Initializes the logging/tracing system.
fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr) // Important: logs go to stderr
        .with_target(false)
        .init();
}

/// Runs Aegis in stdio mode (JSON-RPC over stdin/stdout).
async fn run_stdio_mode(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Aegis in stdio mode");

    let state = Arc::new(RuntimeState::new(config));
    let router = Router::new();
    let mut transport = StdioTransport::new();

    info!("Ready to accept JSON-RPC requests on stdin");

    // Main request loop
    loop {
        match transport.read_request().await {
            Ok(Some(request)) => {
                let response = router.handle(request, state.clone()).await;
                if let Err(e) = transport.write_response(response).await {
                    error!("Failed to write response: {}", e);
                }
            }
            Ok(None) => {
                // EOF - client disconnected
                info!("EOF received, shutting down");
                break;
            }
            Err(e) => {
                error!("Failed to read request: {}", e);
                // For parse errors, we should continue
                // For IO errors, we might want to break
                if matches!(e, aegis::core::AegisError::Io(_)) {
                    break;
                }
            }
        }
    }

    transport.close().await?;
    info!("Aegis stdio mode shut down cleanly");
    Ok(())
}

/// Runs Aegis in HTTP/SSE serve mode.
async fn run_serve_mode(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    use aegis::transport::Metrics;
    
    print_banner(&config);

    let addr = config.socket_addr();
    let state = Arc::new(RuntimeState::new(config.clone()));
    let router = Arc::new(Router::new());
    let metrics = Metrics::new();

    let sse_state = SseState {
        runtime: state,
        router,
        metrics,
    };

    start_server(sse_state, &config, addr).await?;

    Ok(())
}

/// Runs a single tool and exits.
async fn run_oneshot_mode(
    config: Config,
    tool_name: &str,
    args_json: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use aegis::tools::ToolContent;

    let state = Arc::new(RuntimeState::new(config));
    let registry = state.tool_registry.read();

    // Parse arguments
    let arguments: serde_json::Value = serde_json::from_str(args_json)
        .map_err(|e| format!("Invalid JSON arguments: {}", e))?;

    // Execute the tool
    match registry.execute(tool_name, arguments, state.clone()).await {
        Ok(output) => {
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                // Text format - just print the content
                for content in &output.content {
                    match content {
                        ToolContent::Text { text } => println!("{}", text),
                        ToolContent::Image { data, mime_type } => {
                            println!("[Image: {} ({} bytes)]", mime_type, data.len());
                        }
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Lists all available tools.
async fn list_tools(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let extras_enabled = config.extras_enabled;
    let state = Arc::new(RuntimeState::new(config));
    let registry = state.tool_registry.read();

    println!();
    println!("{}", "Available Tools".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    // Core tools list (approximate - these are the core tool names)
    let core_tools: std::collections::HashSet<&str> = [
        "echo", "get_time", "uuid.generate",
        "fs.read_file", "fs.write_file", "cmd.exec",
        "memory.store", "memory.recall", "memory.delete", "memory.list",
        "http.request",
        "env.get", "env.list", "sys.info",
        "base64.encode", "base64.decode",
        "json.parse", "json.query",
        "hash.sha256",
        "regex.match", "regex.replace",
    ].iter().cloned().collect();

    let mut tools: Vec<_> = registry.tools.iter().collect();
    tools.sort_by_key(|(name, _)| *name);

    // Print core tools
    println!();
    println!("  {} {}", "CORE".cyan().bold(), "(always loaded)".dimmed());
    println!("  {}", "─".repeat(40).dimmed());
    for (name, tool) in &tools {
        if core_tools.contains(name.as_str()) {
            let definition = tool.definition();
            println!("  {} {}", "▸".green(), name.white().bold());
            if let Some(desc) = &definition.description {
                println!("    {}", desc.dimmed());
            }
        }
    }

    // Print extra tools if enabled
    if extras_enabled {
        println!();
        println!("  {} {}", "EXTRAS".yellow().bold(), "(optional)".dimmed());
        println!("  {}", "─".repeat(40).dimmed());
        for (name, tool) in &tools {
            if !core_tools.contains(name.as_str()) {
                let definition = tool.definition();
                println!("  {} {}", "▸".yellow(), name.white().bold());
                if let Some(desc) = &definition.description {
                    println!("    {}", desc.dimmed());
                }
            }
        }
    }

    println!();
    println!("{}", "═".repeat(60).cyan());
    
    let core_count = tools.iter().filter(|(n, _)| core_tools.contains(n.as_str())).count();
    let extra_count = tools.len() - core_count;
    
    if extras_enabled && extra_count > 0 {
        println!("  {} core + {} extras = {} tools", 
                 core_count.to_string().green(),
                 extra_count.to_string().yellow(),
                 registry.tools.len().to_string().white().bold());
    } else {
        println!("  {} core tools", core_count.to_string().green());
    }
    println!();

    Ok(())
}

/// Shows server info.
fn show_info(config: &Config) {
    let version = env!("CARGO_PKG_VERSION");

    println!();
    println!("{}", "Aegis Server Info".cyan().bold());
    println!("{}", "═".repeat(40).cyan());
    println!();
    println!("  {} {}", "Version:".dimmed(), version.white());
    println!("  {} {} v{}", "Server:".dimmed(), 
             config.server_name.white(), config.server_version);
    println!("  {} {}:{}", "Bind:".dimmed(), 
             config.host.white(), config.port.to_string().white());
    println!("  {} {}", "Memory:".dimmed(), 
             config.database_path.as_deref().unwrap_or("aegis.db").white());
    println!();
    println!("{}", "Tools".cyan().bold());
    println!("{}", "─".repeat(40).cyan());
    println!("  {} Core tools (always loaded)", "✓".green());
    if config.extras_enabled {
        println!("  {} Extra tools (enabled)", "✓".green());
    } else {
        println!("  {} Extra tools (disabled)", "○".dimmed());
    }
    println!("  {} Custom plugins: {}", "✓".green(), config.plugins.len());
    println!();
    println!("{}", "Capabilities".cyan().bold());
    println!("{}", "─".repeat(40).cyan());
    println!("  {} MCP 2024-11-05 compliant", "✓".green());
    println!("  {} persistent memory (SQLite)", "✓".green());
    println!("  {} stdio & HTTP/SSE transports", "✓".green());
    println!();
}

/// Prints quick start guide.
fn print_quick_start() {
    println!("{}", "Quick Start".cyan().bold());
    println!("{}", "─".repeat(40).cyan());
    println!();
    println!("  {} Run a tool:", "1.".white().bold());
    println!("     {} aegis run echo --args '{{\"text\": \"Hello!\"}}'", "$".dimmed());
    println!();
    println!("  {} Start HTTP server:", "2.".white().bold());
    println!("     {} aegis serve --port 9000", "$".dimmed());
    println!();
    println!("  {} Start stdio mode:", "3.".white().bold());
    println!("     {} aegis --stdio", "$".dimmed());
    println!();
    println!("  {} List tools:", "4.".white().bold());
    println!("     {} aegis tools", "$".dimmed());
    println!();
    println!("Run {} for more options.", "aegis --help".yellow());
    println!();
}

/// Prints the startup banner.
fn print_banner(config: &Config) {
    let version = env!("CARGO_PKG_VERSION");

    eprintln!();
    eprintln!("{}", "╔════════════════════════════════════════════════╗".cyan());
    eprintln!("{}", "║                                                ║".cyan());
    eprintln!("{}{}{}",
        "║       ".cyan(),
        " █████╗ ███████╗ ██████╗ ██╗███████╗".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}{}{}",
        "║       ".cyan(),
        "██╔══██╗██╔════╝██╔════╝ ██║██╔════╝".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}{}{}",
        "║       ".cyan(),
        "███████║█████╗  ██║  ███╗██║███████╗".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}{}{}",
        "║       ".cyan(),
        "██╔══██║██╔══╝  ██║   ██║██║╚════██║".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}{}{}",
        "║       ".cyan(),
        "██║  ██║███████╗╚██████╔╝██║███████║".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}{}{}",
        "║       ".cyan(),
        "╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝".white().bold(),
        "      ║".cyan()
    );
    eprintln!("{}", "║                                                ║".cyan());
    eprintln!("{}{}{}",
        "║         ".cyan(),
        "MCP Tool Server for AI Agents".white(),
        "        ║".cyan()
    );
    eprintln!("{}", "║                                                ║".cyan());
    eprintln!("{}", "╚════════════════════════════════════════════════╝".cyan());
    eprintln!();
    eprintln!("  {} {}", "Version:".dimmed(), version.green());
    eprintln!("  {} {} v{}", "Server:".dimmed(), 
              config.server_name.white(), config.server_version);
    eprintln!("  {} http://{}:{}/mcp", "Endpoint:".dimmed(), 
              config.host.yellow(), config.port.to_string().yellow());
    eprintln!("  {} http://{}:{}/health", "Health:".dimmed(), 
              config.host, config.port);
    eprintln!();
}

