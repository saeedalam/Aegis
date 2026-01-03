//! Transport trait definition.

use async_trait::async_trait;
use crate::core::NexusResult;
use crate::protocol::{Request, Response};

/// Trait defining the interface for MCP message transport.
///
/// Implementations of this trait handle the low-level details of
/// reading requests and writing responses over a specific medium
/// (e.g., stdio, HTTP/SSE).
#[async_trait]
pub trait Transport: Send + Sync {
    /// Reads the next request from the transport.
    ///
    /// Returns `None` when the transport is closed or EOF is reached.
    async fn read_request(&mut self) -> NexusResult<Option<Request>>;

    /// Writes a response to the transport.
    async fn write_response(&mut self, response: Response) -> NexusResult<()>;

    /// Closes the transport gracefully.
    async fn close(&mut self) -> NexusResult<()>;
}


