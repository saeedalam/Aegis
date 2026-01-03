#!/usr/bin/env python3
"""
MCP Client Example - Direct MCP Protocol Communication

This example shows how to communicate with Nexus using the raw MCP protocol
over HTTP. This is useful for understanding the protocol or building custom
integrations.

Run:
    # Terminal 1: Start server
    ./target/release/nexus serve --port 9000
    
    # Terminal 2: Run client
    python3 examples/mcp_client.py
"""
import json
import urllib.request
import urllib.error
from typing import Any, Dict, Optional


class McpClient:
    """A minimal MCP client using HTTP transport (stdlib only)."""
    
    def __init__(self, base_url: str = "http://localhost:9000"):
        self.base_url = base_url
        self.mcp_endpoint = f"{base_url}/mcp"
        self.request_id = 0
        self.initialized = False
    
    def _next_id(self) -> int:
        self.request_id += 1
        return self.request_id
    
    def _send(self, method: str, params: Optional[Dict] = None) -> Dict[str, Any]:
        """Send a JSON-RPC request and return the result."""
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "id": self._next_id()
        }
        if params:
            payload["params"] = params
        
        data = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(
            self.mcp_endpoint,
            data=data,
            headers={"Content-Type": "application/json"}
        )
        
        with urllib.request.urlopen(req) as response:
            result = json.loads(response.read().decode('utf-8'))
        
        if "error" in result:
            raise Exception(f"MCP Error: {result['error']}")
        
        return result.get("result", {})
    
    def initialize(self, client_name: str = "python-mcp-client") -> Dict[str, Any]:
        """Initialize the MCP session."""
        result = self._send("initialize", {
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": client_name,
                "version": "1.0.0"
            },
            "capabilities": {}
        })
        self.initialized = True
        return result
    
    def ping(self) -> Dict[str, Any]:
        """Send a ping request."""
        return self._send("ping")
    
    def list_tools(self) -> Dict[str, Any]:
        """List available tools."""
        return self._send("tools/list")
    
    def call_tool(self, name: str, arguments: Optional[Dict] = None) -> Dict[str, Any]:
        """Call a tool by name."""
        return self._send("tools/call", {
            "name": name,
            "arguments": arguments or {}
        })
    
    def list_resources(self) -> Dict[str, Any]:
        """List available resources."""
        return self._send("resources/list")
    
    def read_resource(self, uri: str) -> Dict[str, Any]:
        """Read a specific resource."""
        return self._send("resources/read", {"uri": uri})
    
    def health_check(self) -> bool:
        """Check if the server is healthy."""
        try:
            with urllib.request.urlopen(f"{self.base_url}/health") as resp:
                return resp.status == 200
        except:
            return False


def main():
    print("=" * 60)
    print("  MCP Client Example")
    print("=" * 60)
    
    client = McpClient("http://localhost:9000")
    
    # 1. Health check
    print("\n📡 Checking server health...")
    if not client.health_check():
        print("❌ Server is not running!")
        print("   Start it with: ./target/release/nexus serve --port 9000")
        return
    print("   ✓ Server is healthy")
    
    # 2. Initialize
    print("\n🤝 Initializing MCP session...")
    init_result = client.initialize("demo-client")
    print(f"   Server: {init_result['serverInfo']['name']} v{init_result['serverInfo']['version']}")
    print(f"   Capabilities: {list(init_result.get('capabilities', {}).keys())}")
    
    # 3. Ping
    print("\n🏓 Sending ping...")
    ping_result = client.ping()
    print(f"   Response: {ping_result}")
    
    # 4. List tools
    print("\n🔧 Listing tools...")
    tools_result = client.list_tools()
    tools = tools_result.get("tools", [])
    print(f"   Found {len(tools)} tools:")
    for tool in tools[:5]:  # Show first 5
        print(f"   - {tool['name']}: {tool.get('description', 'No description')[:50]}...")
    if len(tools) > 5:
        print(f"   ... and {len(tools) - 5} more")
    
    # 5. Call tools
    print("\n⚡ Calling tools...")
    
    # Echo
    echo_result = client.call_tool("echo", {"text": "Hello from MCP!"})
    content = echo_result.get("content", [{}])[0].get("text", "")
    print(f"   echo: {content}")
    
    # Get time
    time_result = client.call_tool("get_time", {})
    time_content = time_result.get("content", [{}])[0].get("text", "")
    time_data = json.loads(time_content)
    print(f"   get_time: {time_data.get('time', 'unknown')}")
    
    # Memory operations
    client.call_tool("memory.store", {"key": "mcp_test", "value": "it works!"})
    recall_result = client.call_tool("memory.recall", {"key": "mcp_test"})
    recall_content = recall_result.get("content", [{}])[0].get("text", "")
    recall_data = json.loads(recall_content)
    print(f"   memory.recall: {recall_data.get('value', 'not found')}")
    
    # 6. List resources
    print("\n📚 Listing resources...")
    resources_result = client.list_resources()
    resources = resources_result.get("resources", [])
    print(f"   Found {len(resources)} resources:")
    for resource in resources:
        print(f"   - {resource['uri']}: {resource.get('name', 'unnamed')}")
    
    # 7. Read a resource
    print("\n📖 Reading KV store resource...")
    try:
        kv_result = client.read_resource("nexus://kv")
        contents = kv_result.get("contents", [])
        if contents and isinstance(contents, list) and len(contents) > 0:
            content_item = contents[0]
            if isinstance(content_item, dict):
                kv_content = content_item.get("text", "{}")
                kv_data = json.loads(kv_content)
                print(f"   Keys in store: {kv_data.get('count', 0)}")
                for item in kv_data.get("items", [])[:3]:
                    val = str(item.get('value', ''))[:30]
                    print(f"   - {item['key']}: {val}...")
            else:
                print(f"   Content: {contents}")
        else:
            print(f"   Raw result: {kv_result}")
    except Exception as e:
        print(f"   Could not read resource: {e}")
    
    print("\n" + "=" * 60)
    print("  MCP Client Example Complete!")
    print("=" * 60)


if __name__ == "__main__":
    try:
        main()
    except urllib.error.URLError:
        print("❌ Could not connect to Nexus server.")
        print("   Make sure to start it first:")
        print("   ./target/release/nexus serve --port 9000")
    except Exception as e:
        print(f"❌ Error: {e}")

