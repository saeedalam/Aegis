"""Nexus MCP Client - A minimal Python client for Nexus.

Usage:
    from nexus_client import NexusClient
    
    client = NexusClient()
    result = client.call("echo", {"text": "Hello!"})
    print(result)
"""
import json
import subprocess
from typing import Any, Dict, Optional
from pathlib import Path


class NexusError(Exception):
    """Error from Nexus execution."""
    pass


class NexusClient:
    """Minimal client for the Nexus MCP runtime."""
    
    def __init__(self, binary_path: str = "nexus"):
        """Initialize the Nexus client.
        
        Args:
            binary_path: Path to the nexus binary (default: 'nexus' in PATH)
        """
        self.binary_path = binary_path
    
    def call(self, tool: str, args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Call a Nexus tool.
        
        Args:
            tool: The tool name to execute
            args: Optional dictionary of arguments
            
        Returns:
            Dictionary containing the tool output
            
        Raises:
            NexusError: If tool execution fails
        """
        args = args or {}
        args_json = json.dumps(args)
        
        result = subprocess.run(
            [self.binary_path, "run", tool, "--args", args_json, "--format", "json"],
            capture_output=True,
            text=True,
        )
        
        if result.returncode != 0:
            raise NexusError(result.stderr.strip() or f"Tool '{tool}' failed")
        
        return json.loads(result.stdout)
    
    def call_text(self, tool: str, args: Optional[Dict[str, Any]] = None) -> str:
        """Call a Nexus tool and return text output.
        
        Args:
            tool: The tool name to execute
            args: Optional dictionary of arguments
            
        Returns:
            Text output from the tool
        """
        output = self.call(tool, args)
        return output.get("content", [{}])[0].get("text", "")
    
    def list_tools(self) -> str:
        """List available tools."""
        result = subprocess.run(
            [self.binary_path, "tools"],
            capture_output=True,
            text=True,
        )
        return result.stdout
    
    def memory_store(self, key: str, value: str) -> bool:
        """Store a value in persistent memory."""
        self.call("memory.store", {"key": key, "value": value})
        return True
    
    def memory_recall(self, key: str) -> Optional[str]:
        """Recall a value from persistent memory."""
        try:
            text = self.call_text("memory.recall", {"key": key})
            # Parse JSON response
            data = json.loads(text)
            if data.get("found"):
                return data.get("value")
            return None
        except (NexusError, json.JSONDecodeError):
            return None
    
    def memory_list(self, prefix: Optional[str] = None) -> list:
        """List all memory keys."""
        args = {"prefix": prefix} if prefix else {}
        text = self.call_text("memory.list", args)
        try:
            data = json.loads(text)
            return data.get("keys", [])
        except json.JSONDecodeError:
            return []

