//! Web dashboard for Nexus monitoring and management.

use axum::{
    extract::State,
    response::Html,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::core::RuntimeState;

/// Dashboard routes.
pub fn dashboard_routes(state: Arc<RuntimeState>) -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/api/stats", get(stats_api))
        .route("/api/tools", get(tools_api))
        .route("/api/memory", get(memory_api))
        .route("/api/secrets", get(secrets_api))
        .route("/api/tasks", get(tasks_api))
        .with_state(state)
}

/// Main dashboard HTML page.
async fn dashboard_page() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

/// Stats API response.
#[derive(Serialize)]
struct StatsResponse {
    server_name: String,
    server_version: String,
    tools_count: usize,
    secrets_count: usize,
    tasks_count: usize,
    initialized: bool,
}

/// Stats API handler.
async fn stats_api(State(state): State<Arc<RuntimeState>>) -> Json<StatsResponse> {
    Json(StatsResponse {
        server_name: state.server_info.name.clone(),
        server_version: state.server_info.version.clone(),
        tools_count: state.tool_registry.read().list_definitions().len(),
        secrets_count: state.secrets.list().len(),
        tasks_count: state.scheduler.list_tasks().len(),
        initialized: state.is_initialized(),
    })
}

/// Tool info for API.
#[derive(Serialize)]
struct ToolInfo {
    name: String,
    description: Option<String>,
}

/// Tools API handler.
async fn tools_api(State(state): State<Arc<RuntimeState>>) -> Json<Vec<ToolInfo>> {
    let tools = state
        .tool_registry
        .read()
        .list_definitions()
        .into_iter()
        .map(|def| ToolInfo {
            name: def.name,
            description: def.description,
        })
        .collect();

    Json(tools)
}

/// Memory stats for API.
#[derive(Serialize)]
struct MemoryStats {
    kv_keys: Vec<String>,
}

/// Memory API handler.
async fn memory_api(State(state): State<Arc<RuntimeState>>) -> Json<MemoryStats> {
    let keys = state.memory_store.kv_list(None).await.unwrap_or_default();
    Json(MemoryStats { kv_keys: keys })
}

/// Secrets list for API (keys only).
#[derive(Serialize)]
struct SecretsStats {
    keys: Vec<String>,
}

/// Secrets API handler.
async fn secrets_api(State(state): State<Arc<RuntimeState>>) -> Json<SecretsStats> {
    Json(SecretsStats {
        keys: state.secrets.list(),
    })
}

/// Task info for API.
#[derive(Serialize)]
struct TaskInfo {
    id: String,
    name: String,
    cron: String,
    tool: String,
    enabled: bool,
    last_run: Option<String>,
}

/// Tasks API handler.
async fn tasks_api(State(state): State<Arc<RuntimeState>>) -> Json<Vec<TaskInfo>> {
    let tasks = state
        .scheduler
        .list_tasks()
        .iter()
        .map(|t| TaskInfo {
            id: t.id.clone(),
            name: t.name.clone(),
            cron: t.cron.clone(),
            tool: t.tool.clone(),
            enabled: t.enabled,
            last_run: t.last_run.clone(),
        })
        .collect();

    Json(tasks)
}

/// Embedded dashboard HTML.
const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nexus Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        :root {
            --bg-primary: #0a0a0f;
            --bg-secondary: #12121a;
            --bg-card: #1a1a24;
            --text-primary: #e8e8f0;
            --text-secondary: #8888a0;
            --accent: #7c3aed;
            --accent-glow: rgba(124, 58, 237, 0.3);
            --success: #22c55e;
            --warning: #f59e0b;
            --error: #ef4444;
            --border: #2a2a3a;
        }
        
        body {
            font-family: 'SF Mono', 'JetBrains Mono', 'Fira Code', monospace;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
        }
        
        header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border);
        }
        
        .logo {
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        
        .logo h1 {
            font-size: 2rem;
            font-weight: 700;
            background: linear-gradient(135deg, var(--accent), #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        
        .logo-icon {
            width: 48px;
            height: 48px;
            background: linear-gradient(135deg, var(--accent), #a78bfa);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.5rem;
            box-shadow: 0 0 30px var(--accent-glow);
        }
        
        .status-badge {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 1rem;
            background: var(--bg-card);
            border-radius: 9999px;
            border: 1px solid var(--border);
        }
        
        .status-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: var(--success);
            animation: pulse 2s infinite;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }
        
        .card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 1.5rem;
            transition: all 0.3s ease;
        }
        
        .card:hover {
            border-color: var(--accent);
            box-shadow: 0 0 30px var(--accent-glow);
        }
        
        .card-header {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 1rem;
        }
        
        .card-icon {
            width: 40px;
            height: 40px;
            background: var(--bg-secondary);
            border-radius: 10px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.25rem;
        }
        
        .card-title {
            font-size: 0.875rem;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.1em;
        }
        
        .card-value {
            font-size: 2.5rem;
            font-weight: 700;
            color: var(--text-primary);
        }
        
        .section {
            margin-bottom: 2rem;
        }
        
        .section-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 1rem;
        }
        
        .section-title {
            font-size: 1.25rem;
            font-weight: 600;
        }
        
        .list {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            overflow: hidden;
        }
        
        .list-item {
            padding: 1rem 1.5rem;
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            transition: background 0.2s ease;
        }
        
        .list-item:last-child {
            border-bottom: none;
        }
        
        .list-item:hover {
            background: var(--bg-secondary);
        }
        
        .list-item-name {
            font-weight: 500;
            color: var(--accent);
        }
        
        .list-item-desc {
            font-size: 0.875rem;
            color: var(--text-secondary);
            margin-top: 0.25rem;
        }
        
        .tag {
            padding: 0.25rem 0.75rem;
            background: var(--bg-secondary);
            border-radius: 9999px;
            font-size: 0.75rem;
            color: var(--text-secondary);
        }
        
        .tag.enabled {
            background: rgba(34, 197, 94, 0.2);
            color: var(--success);
        }
        
        .tag.disabled {
            background: rgba(239, 68, 68, 0.2);
            color: var(--error);
        }
        
        .empty-state {
            text-align: center;
            padding: 3rem;
            color: var(--text-secondary);
        }
        
        .loading {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 3rem;
        }
        
        .spinner {
            width: 40px;
            height: 40px;
            border: 3px solid var(--border);
            border-top-color: var(--accent);
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
        
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        
        footer {
            text-align: center;
            padding: 2rem;
            color: var(--text-secondary);
            font-size: 0.875rem;
            border-top: 1px solid var(--border);
            margin-top: 2rem;
        }
        
        footer a {
            color: var(--accent);
            text-decoration: none;
        }
        
        footer a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <div class="logo-icon">⚡</div>
                <h1>Nexus</h1>
            </div>
            <div class="status-badge">
                <span class="status-dot"></span>
                <span id="status-text">Running</span>
            </div>
        </header>
        
        <div class="grid" id="stats-grid">
            <div class="card">
                <div class="card-header">
                    <div class="card-icon">🔧</div>
                    <span class="card-title">Tools</span>
                </div>
                <div class="card-value" id="tools-count">-</div>
            </div>
            <div class="card">
                <div class="card-header">
                    <div class="card-icon">🔐</div>
                    <span class="card-title">Secrets</span>
                </div>
                <div class="card-value" id="secrets-count">-</div>
            </div>
            <div class="card">
                <div class="card-header">
                    <div class="card-icon">⏰</div>
                    <span class="card-title">Tasks</span>
                </div>
                <div class="card-value" id="tasks-count">-</div>
            </div>
            <div class="card">
                <div class="card-header">
                    <div class="card-icon">💾</div>
                    <span class="card-title">Memory Keys</span>
                </div>
                <div class="card-value" id="memory-count">-</div>
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">🔧 Available Tools</h2>
            </div>
            <div class="list" id="tools-list">
                <div class="loading"><div class="spinner"></div></div>
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">⏰ Scheduled Tasks</h2>
            </div>
            <div class="list" id="tasks-list">
                <div class="loading"><div class="spinner"></div></div>
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">🔐 Stored Secrets</h2>
            </div>
            <div class="list" id="secrets-list">
                <div class="loading"><div class="spinner"></div></div>
            </div>
        </div>
        
        <footer>
            <p>Nexus MCP Runtime &bull; <a href="https://github.com/your-org/nexus">GitHub</a></p>
        </footer>
    </div>
    
    <script>
        async function fetchData() {
            try {
                // Fetch stats
                const statsRes = await fetch('/dashboard/api/stats');
                const stats = await statsRes.json();
                document.getElementById('tools-count').textContent = stats.tools_count;
                document.getElementById('secrets-count').textContent = stats.secrets_count;
                document.getElementById('tasks-count').textContent = stats.tasks_count;
                document.getElementById('status-text').textContent = 
                    stats.initialized ? 'Initialized' : 'Running';
                
                // Fetch memory
                const memoryRes = await fetch('/dashboard/api/memory');
                const memory = await memoryRes.json();
                document.getElementById('memory-count').textContent = memory.kv_keys.length;
                
                // Fetch tools
                const toolsRes = await fetch('/dashboard/api/tools');
                const tools = await toolsRes.json();
                renderTools(tools);
                
                // Fetch tasks
                const tasksRes = await fetch('/dashboard/api/tasks');
                const tasks = await tasksRes.json();
                renderTasks(tasks);
                
                // Fetch secrets
                const secretsRes = await fetch('/dashboard/api/secrets');
                const secrets = await secretsRes.json();
                renderSecrets(secrets);
            } catch (error) {
                console.error('Failed to fetch data:', error);
            }
        }
        
        function renderTools(tools) {
            const list = document.getElementById('tools-list');
            if (tools.length === 0) {
                list.innerHTML = '<div class="empty-state">No tools available</div>';
                return;
            }
            list.innerHTML = tools.map(tool => `
                <div class="list-item">
                    <div>
                        <div class="list-item-name">${tool.name}</div>
                        <div class="list-item-desc">${tool.description || 'No description'}</div>
                    </div>
                </div>
            `).join('');
        }
        
        function renderTasks(tasks) {
            const list = document.getElementById('tasks-list');
            if (tasks.length === 0) {
                list.innerHTML = '<div class="empty-state">No scheduled tasks</div>';
                return;
            }
            list.innerHTML = tasks.map(task => `
                <div class="list-item">
                    <div>
                        <div class="list-item-name">${task.name}</div>
                        <div class="list-item-desc">${task.cron} → ${task.tool}</div>
                    </div>
                    <span class="tag ${task.enabled ? 'enabled' : 'disabled'}">
                        ${task.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                </div>
            `).join('');
        }
        
        function renderSecrets(secrets) {
            const list = document.getElementById('secrets-list');
            if (secrets.keys.length === 0) {
                list.innerHTML = '<div class="empty-state">No secrets stored</div>';
                return;
            }
            list.innerHTML = secrets.keys.map(key => `
                <div class="list-item">
                    <div class="list-item-name">${key}</div>
                    <span class="tag">••••••••</span>
                </div>
            `).join('');
        }
        
        // Initial fetch
        fetchData();
        
        // Refresh every 5 seconds
        setInterval(fetchData, 5000);
    </script>
</body>
</html>
"##;

