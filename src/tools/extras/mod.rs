//! Extra tools - Optional capabilities that extend Nexus.
//!
//! These tools are opinionated and provide higher-level functionality.
//! They can be enabled via configuration or feature flags.
//!
//! Categories:
//! - llm: LLM provider integrations (OpenAI, Anthropic)
//! - vector: Vector storage and semantic search
//! - git: Git repository operations
//! - notify: Notifications (Slack, Discord, Email, Webhooks)
//! - workflow: Workflow/pipeline orchestration
//! - scheduler: Cron-like task scheduling
//! - web: Web scraping and search
//! - conversation: Conversation history management
//! - secrets: Secure credential storage

mod llm;
mod vector;
mod git;
mod notify;
mod workflow;
mod scheduler;
mod web;
mod conversation;
mod secrets;

use std::sync::Arc;
use tracing::info;
use crate::tools::ToolRegistry;
use crate::core::Config;

pub use llm::{OpenAiChatTool, AnthropicChatTool, EmbeddingsTool};
pub use vector::{VectorStoreTool, VectorSearchTool, VectorDeleteTool, VectorListTool};
pub use git::{GitStatusTool, GitLogTool, GitDiffTool, GitCommitTool, GitBranchTool};
pub use notify::{WebhookSendTool, SlackNotifyTool, DiscordNotifyTool, EmailNotifyTool};
pub use workflow::{WorkflowRunTool, WorkflowDefineTool, WorkflowExecuteTool, WorkflowListTool};
pub use scheduler::{SchedulerCreateTool, SchedulerListTool, SchedulerDeleteTool, SchedulerToggleTool, SchedulerRunTool};
pub use web::{WebExtractTool, WebSearchTool};
pub use conversation::{ConversationCreateTool, ConversationAddTool, ConversationGetTool, ConversationListTool, ConversationSearchTool};
pub use secrets::{SecretsSetTool, SecretsGetTool, SecretsListTool, SecretsDeleteTool};

/// Registers all extra tools with the registry.
/// Call this only if extras are enabled in config.
pub fn register_extra_tools(registry: &mut ToolRegistry, _config: &Config) {
    info!("Loading extra tools...");

    // LLM integration tools
    registry.register(Arc::new(OpenAiChatTool));
    registry.register(Arc::new(AnthropicChatTool));
    registry.register(Arc::new(EmbeddingsTool));

    // Vector store tools
    registry.register(Arc::new(VectorStoreTool));
    registry.register(Arc::new(VectorSearchTool));
    registry.register(Arc::new(VectorDeleteTool));
    registry.register(Arc::new(VectorListTool));

    // Git tools
    registry.register(Arc::new(GitStatusTool));
    registry.register(Arc::new(GitLogTool));
    registry.register(Arc::new(GitDiffTool));
    registry.register(Arc::new(GitCommitTool));
    registry.register(Arc::new(GitBranchTool));

    // Notification tools
    registry.register(Arc::new(WebhookSendTool));
    registry.register(Arc::new(SlackNotifyTool));
    registry.register(Arc::new(DiscordNotifyTool));
    registry.register(Arc::new(EmailNotifyTool));

    // Workflow tools
    registry.register(Arc::new(WorkflowRunTool));
    registry.register(Arc::new(WorkflowDefineTool));
    registry.register(Arc::new(WorkflowExecuteTool));
    registry.register(Arc::new(WorkflowListTool));

    // Scheduler tools
    registry.register(Arc::new(SchedulerCreateTool));
    registry.register(Arc::new(SchedulerListTool));
    registry.register(Arc::new(SchedulerDeleteTool));
    registry.register(Arc::new(SchedulerToggleTool));
    registry.register(Arc::new(SchedulerRunTool));

    // Web tools
    registry.register(Arc::new(WebExtractTool));
    registry.register(Arc::new(WebSearchTool));

    // Conversation tools
    registry.register(Arc::new(ConversationCreateTool));
    registry.register(Arc::new(ConversationAddTool));
    registry.register(Arc::new(ConversationGetTool));
    registry.register(Arc::new(ConversationListTool));
    registry.register(Arc::new(ConversationSearchTool));

    // Secrets tools
    registry.register(Arc::new(SecretsSetTool));
    registry.register(Arc::new(SecretsGetTool));
    registry.register(Arc::new(SecretsListTool));
    registry.register(Arc::new(SecretsDeleteTool));

    info!("Loaded {} extra tools", extra_tool_count());
}

/// Returns the count of extra tools.
pub fn extra_tool_count() -> usize {
    39 // 3 llm + 4 vector + 5 git + 4 notify + 4 workflow + 5 scheduler + 2 web + 5 conversation + 4 secrets + 3 (script plugins counted separately)
}


