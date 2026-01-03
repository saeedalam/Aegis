//! Vector storage and search tools for semantic search.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::core::RuntimeState;
use crate::protocol::mcp::Tool as ToolDefinition;
use crate::tools::registry::{Tool, ToolError, ToolOutput};

/// Tool to store a vector embedding.
#[derive(Debug)]
pub struct VectorStoreTool;

#[async_trait]
impl Tool for VectorStoreTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "vector.store".to_string(),
            description: Some(
                "Stores a text with its vector embedding for semantic search. Use with llm.embed to generate embeddings."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier for this vector"
                    },
                    "text": {
                        "type": "string",
                        "description": "The text content"
                    },
                    "embedding": {
                        "type": "array",
                        "items": {"type": "number"},
                        "description": "Vector embedding (from llm.embed)"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Namespace/collection (default: 'default')"
                    }
                },
                "required": ["id", "embedding"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'id'".to_string()))?;

        let embedding = arguments
            .get("embedding")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'embedding' array".to_string()))?;

        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let metadata = arguments.get("metadata").cloned().unwrap_or(json!({}));

        let namespace = arguments
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Convert embedding to f64 vec
        let embedding_vec: Vec<f64> = embedding
            .iter()
            .filter_map(|v| v.as_f64())
            .collect();

        if embedding_vec.is_empty() {
            return Err(ToolError::InvalidInput("Empty embedding".to_string()));
        }

        // Store in memory store
        let key = format!("vector:{}:{}", namespace, id);
        let value = json!({
            "id": id,
            "text": text,
            "embedding": embedding_vec,
            "metadata": metadata,
            "namespace": namespace,
            "dimensions": embedding_vec.len()
        });

        state
            .memory_store
            .kv_set(&key, value, None)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let result = json!({
            "success": true,
            "id": id,
            "namespace": namespace,
            "dimensions": embedding_vec.len()
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to search vectors by similarity.
#[derive(Debug)]
pub struct VectorSearchTool;

#[async_trait]
impl Tool for VectorSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "vector.search".to_string(),
            description: Some(
                "Searches stored vectors by similarity. Returns most similar results."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "embedding": {
                        "type": "array",
                        "items": {"type": "number"},
                        "description": "Query embedding (from llm.embed)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default: 5)"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Namespace to search (default: 'default')"
                    },
                    "threshold": {
                        "type": "number",
                        "description": "Minimum similarity score 0-1 (default: 0)"
                    }
                },
                "required": ["embedding"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let query_embedding = arguments
            .get("embedding")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'embedding' array".to_string()))?;

        let query_vec: Vec<f64> = query_embedding
            .iter()
            .filter_map(|v| v.as_f64())
            .collect();

        if query_vec.is_empty() {
            return Err(ToolError::InvalidInput("Empty embedding".to_string()));
        }

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let namespace = arguments
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let threshold = arguments
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Get all vectors in namespace
        let prefix = format!("vector:{}:", namespace);
        let keys = state
            .memory_store
            .kv_list(Some(&prefix))
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let mut results: Vec<(f64, Value)> = Vec::new();

        for key in keys {
            if let Ok(Some(kv)) = state.memory_store.kv_get(&key).await {
                if let Some(stored_embedding) = kv.value.get("embedding").and_then(|v| v.as_array()) {
                    let stored_vec: Vec<f64> = stored_embedding
                        .iter()
                        .filter_map(|v| v.as_f64())
                        .collect();

                    if stored_vec.len() == query_vec.len() {
                        let similarity = cosine_similarity(&query_vec, &stored_vec);

                        if similarity >= threshold {
                            let mut result = kv.value.clone();
                            result["score"] = json!(similarity);
                            // Remove embedding from result to save space
                            if let Some(obj) = result.as_object_mut() {
                                obj.remove("embedding");
                            }
                            results.push((similarity, result));
                        }
                    }
                }
            }
        }

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let top_results: Vec<Value> = results
            .into_iter()
            .take(limit)
            .map(|(_, v)| v)
            .collect();

        let output = json!({
            "namespace": namespace,
            "count": top_results.len(),
            "results": top_results
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&output).unwrap()))
    }
}

/// Tool to delete a vector.
#[derive(Debug)]
pub struct VectorDeleteTool;

#[async_trait]
impl Tool for VectorDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "vector.delete".to_string(),
            description: Some("Deletes a stored vector.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Vector ID to delete"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Namespace (default: 'default')"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'id'".to_string()))?;

        let namespace = arguments
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let key = format!("vector:{}:{}", namespace, id);

        state
            .memory_store
            .kv_delete(&key)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let result = json!({
            "success": true,
            "id": id,
            "namespace": namespace
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Tool to list vectors in a namespace.
#[derive(Debug)]
pub struct VectorListTool;

#[async_trait]
impl Tool for VectorListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "vector.list".to_string(),
            description: Some("Lists all vectors in a namespace.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "namespace": {
                        "type": "string",
                        "description": "Namespace (default: 'default')"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        arguments: Value,
        state: Arc<RuntimeState>,
    ) -> Result<ToolOutput, ToolError> {
        let namespace = arguments
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let prefix = format!("vector:{}:", namespace);
        let keys = state
            .memory_store
            .kv_list(Some(&prefix))
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let vectors: Vec<Value> = keys
            .iter()
            .filter_map(|k| k.strip_prefix(&prefix))
            .map(|id| json!({"id": id}))
            .collect();

        let result = json!({
            "namespace": namespace,
            "count": vectors.len(),
            "vectors": vectors
        });

        Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap()))
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}


