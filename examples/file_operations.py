#!/usr/bin/env python3
"""File Operations Example - Reading and writing files via Nexus.

This example demonstrates using Nexus's fs tools for file operations.
Note: File operations are restricted to configured allowed paths.

Usage:
    python3 examples/file_operations.py
"""
import sys
import json
import tempfile
import os

sys.path.insert(0, "sdk/python")
from nexus_client import NexusClient, NexusError


def main():
    print("=" * 50)
    print("📁 File Operations Example")
    print("=" * 50)
    
    client = NexusClient()
    
    # Use temp directory for safe testing
    with tempfile.TemporaryDirectory() as tmpdir:
        test_file = os.path.join(tmpdir, "test.txt")
        
        print(f"\n📝 Writing to: {test_file}")
        
        # Note: fs.write_file requires path to be in allowed_write_paths
        # For this example, you may need to configure nexus.json
        try:
            # Write a file
            result = client.call("fs.write_file", {
                "path": test_file,
                "content": "Hello from Nexus!\nThis file was created by an AI agent."
            })
            print(f"   Write result: {result}")
            
            # Read it back
            print(f"\n📖 Reading from: {test_file}")
            content = client.call_text("fs.read_file", {"path": test_file})
            print(f"   Content:\n   {content.replace(chr(10), chr(10) + '   ')}")
            
        except NexusError as e:
            print(f"\n⚠️  File operation restricted: {e}")
            print("\nTo enable file operations, configure nexus.json:")
            print(json.dumps({
                "security": {
                    "allowed_read_paths": ["/tmp", os.getcwd()],
                    "allowed_write_paths": ["/tmp"]
                }
            }, indent=2))
    
    print("\n" + "=" * 50)
    print("✅ Example complete!")
    print("=" * 50)


if __name__ == "__main__":
    try:
        main()
    except FileNotFoundError:
        print("❌ Error: 'nexus' binary not found.")
        print("   Build: cargo build --release")
        print("   Add to PATH: export PATH=\"$PATH:$(pwd)/target/release\"")
        sys.exit(1)


