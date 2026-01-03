//! Stdio transport implementation.
//!
//! This transport reads JSON-RPC requests from stdin (one per line)
//! and writes responses to stdout. Logs go to stderr to avoid
//! corrupting the JSON-RPC stream.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use tracing::{debug, trace};

use crate::core::{NexusError, NexusResult};
use crate::protocol::{Request, Response};
use crate::transport::Transport;

/// Stdio-based transport for MCP communication.
///
/// Reads newline-delimited JSON-RPC requests from stdin and writes
/// responses to stdout. This is the standard transport for CLI-based
/// MCP clients.
pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: Stdout,
    buffer: String,
}

impl StdioTransport {
    /// Creates a new StdioTransport using tokio's stdin/stdout.
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
            buffer: String::with_capacity(4096),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn read_request(&mut self) -> NexusResult<Option<Request>> {
        self.buffer.clear();

        // Read a line from stdin
        let bytes_read = self.reader.read_line(&mut self.buffer).await?;

        // EOF check
        if bytes_read == 0 {
            debug!("EOF reached on stdin");
            return Ok(None);
        }

        // Trim whitespace
        let line = self.buffer.trim();

        // Skip empty lines
        if line.is_empty() {
            trace!("Skipping empty line");
            return self.read_request().await;
        }

        trace!("Received line: {}", line);

        // Parse JSON-RPC request
        let request: Request = serde_json::from_str(line).map_err(|e| {
            NexusError::JsonParse(e)
        })?;

        // Validate the request
        request.validate()?;

        debug!("Parsed request: method={}, id={:?}", request.method, request.id);

        Ok(Some(request))
    }

    async fn write_response(&mut self, response: Response) -> NexusResult<()> {
        let json = response.to_json()?;

        trace!("Sending response: {}", json);

        // Write JSON followed by newline
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        debug!("Response sent for id={:?}", response.id);

        Ok(())
    }

    async fn close(&mut self) -> NexusResult<()> {
        debug!("Closing stdio transport");
        self.writer.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_creation() {
        // Just verify we can create the transport
        let _transport = StdioTransport::new();
    }
}


