#!/usr/bin/env python3
"""Hello Nexus - A simple example using the Nexus MCP runtime.

This example demonstrates:
1. Calling the echo tool
2. Getting the current time
3. Using persistent memory

Make sure nexus is running or in your PATH:
    cargo build --release
    export PATH="$PATH:$(pwd)/target/release"
    python3 examples/hello_nexus.py
"""
import sys
sys.path.insert(0, "sdk/python")

from nexus_client import NexusClient, NexusError


def main():
    print("🚀 Hello Nexus Example")
    print("=" * 40)
    
    # Create client (assumes 'nexus' is in PATH)
    client = NexusClient()
    
    # 1. Echo tool
    print("\n📢 Testing echo tool...")
    result = client.call_text("echo", {"text": "Hello from Python!"})
    print(f"   Echo: {result}")
    
    # 2. Get time
    print("\n⏰ Getting current time...")
    time_result = client.call_text("get_time", {})
    print(f"   Time: {time_result}")
    
    # 3. Memory operations
    print("\n💾 Testing persistent memory...")
    
    # Store a value
    client.memory_store("user_name", "Claude")
    print("   Stored: user_name = 'Claude'")
    
    # Recall it
    name = client.memory_recall("user_name")
    print(f"   Recalled: user_name = '{name}'")
    
    # List all keys
    keys = client.memory_list()
    print(f"   All keys: {keys}")
    
    print("\n✅ All tests passed!")
    print("=" * 40)


if __name__ == "__main__":
    try:
        main()
    except NexusError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)
    except FileNotFoundError:
        print("❌ Error: 'nexus' binary not found. Build it with:")
        print("   cargo build --release")
        print("   export PATH=\"$PATH:$(pwd)/target/release\"")
        sys.exit(1)


