//! Runtime state management for Nexus.

use crate::core::Config;
use crate::memory::{MemoryStore, SqliteStore};
use crate::protocol::mcp::{ResourcesCapability, ServerCapabilities, ServerInfo};
use crate::scheduler::Scheduler;
use crate::secrets::SecretsManager;
use crate::tools::{register_core_tools, register_extra_tools, ToolRegistry};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Shared runtime state for the Nexus server.
#[derive(Debug)]
pub struct RuntimeState {
    /// Server configuration.
    pub config: Config,

    /// Whether the server has been initialized via MCP handshake.
    initialized: AtomicBool,

    /// Server capabilities advertised to clients.
    pub capabilities: ServerCapabilities,

    /// Server information.
    pub server_info: ServerInfo,

    /// Tool registry for executing tools.
    pub tool_registry: RwLock<ToolRegistry>,

    /// Memory store for persistent storage.
    pub memory_store: Arc<dyn MemoryStore>,

    /// Secrets manager for secure credential storage.
    pub secrets: Arc<SecretsManager>,

    /// Task scheduler for automated execution.
    pub scheduler: Arc<Scheduler>,
}

impl RuntimeState {
    /// Creates a new runtime state with the given configuration.
    pub fn new(config: Config) -> Self {
        let server_info = ServerInfo {
            name: config.server_name.clone(),
            version: config.server_version.clone(),
        };

        // Create and populate tool registry
        let mut tool_registry = ToolRegistry::new();
        
        // Always register core tools
        register_core_tools(&mut tool_registry, &config);
        info!("Loaded {} core tools", crate::tools::core::core_tool_count());
        
        // Register extra tools if enabled
        if config.extras_enabled {
            register_extra_tools(&mut tool_registry, &config);
        } else {
            info!("Extra tools disabled (enable with extras_enabled: true in config)");
        }

        // Create memory store
        let db_path = config.database_path.clone().unwrap_or_else(|| "aegis.db".to_string());
        let memory_store: Arc<dyn MemoryStore> = match SqliteStore::new(&db_path) {
            Ok(store) => {
                info!("Memory store initialized at: {}", db_path);
                Arc::new(store)
            }
            Err(e) => {
                tracing::warn!("Failed to create SQLite store: {}, using in-memory", e);
                Arc::new(SqliteStore::in_memory().expect("Failed to create in-memory store"))
            }
        };

        // Create secrets manager
        let secrets_path = config
            .database_path
            .as_ref()
            .map(|p| p.replace(".db", ".secrets"));
        let secrets = Arc::new(SecretsManager::new(secrets_path, None));
        info!("Secrets manager initialized");

        // Create scheduler
        let scheduler = Arc::new(Scheduler::new());
        info!("Scheduler initialized");

        // Build capabilities with resources enabled
        let capabilities = ServerCapabilities {
            tools: Some(crate::protocol::mcp::ToolsCapability { list_changed: false }),
            prompts: Some(crate::protocol::mcp::PromptsCapability { list_changed: false }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
        };

        Self {
            config,
            initialized: AtomicBool::new(false),
            capabilities,
            server_info,
            tool_registry: RwLock::new(tool_registry),
            memory_store,
            secrets,
            scheduler,
        }
    }

    /// Returns whether the server has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    /// Marks the server as initialized.
    pub fn set_initialized(&self) {
        self.initialized.store(true, Ordering::SeqCst);
    }

    /// Creates a shared reference to the runtime state.
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}
