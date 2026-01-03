#!/usr/bin/env python3
"""Agent Memory Example - Demonstrating persistent memory across sessions.

This example shows how an AI agent can maintain state between sessions
using Nexus's built-in memory system.

Usage:
    python3 examples/agent_memory.py
"""
import sys
import json
from datetime import datetime

sys.path.insert(0, "sdk/python")
from nexus_client import NexusClient, NexusError


def simulate_agent_session(client: NexusClient, session_id: int):
    """Simulate an agent session that remembers past interactions."""
    
    print(f"\n🤖 Agent Session {session_id}")
    print("-" * 40)
    
    # Check for previous sessions
    visit_count = client.memory_recall("visit_count")
    if visit_count:
        count = int(visit_count) + 1
        print(f"   Welcome back! This is visit #{count}")
    else:
        count = 1
        print("   First visit! Nice to meet you.")
    
    # Update visit count
    client.memory_store("visit_count", str(count))
    
    # Store last visit time
    now = datetime.now().isoformat()
    last_visit = client.memory_recall("last_visit")
    if last_visit:
        print(f"   Last visit: {last_visit}")
    
    client.memory_store("last_visit", now)
    print(f"   Current time: {now}")
    
    # Store session notes
    note_key = f"session_{session_id}_notes"
    client.memory_store(note_key, json.dumps({
        "session_id": session_id,
        "timestamp": now,
        "message": f"Session {session_id} completed successfully"
    }))
    print(f"   Session notes saved to: {note_key}")
    
    # List all memory keys
    keys = client.memory_list()
    print(f"\n   📚 Memory contains {len(keys)} entries: {', '.join(keys)}")


def main():
    print("=" * 50)
    print("🧠 Agent Memory Example")
    print("=" * 50)
    print("\nThis example simulates multiple agent sessions,")
    print("demonstrating how memory persists between runs.")
    
    client = NexusClient()
    
    # Simulate 3 sessions
    for i in range(1, 4):
        simulate_agent_session(client, i)
    
    print("\n" + "=" * 50)
    print("✅ Example complete!")
    print("\nTry running this script again - the memory persists!")
    print("=" * 50)


if __name__ == "__main__":
    try:
        main()
    except NexusError as e:
        print(f"❌ Error: {e}")
        sys.exit(1)
    except FileNotFoundError:
        print("❌ Error: 'nexus' binary not found.")
        print("   Build: cargo build --release")
        print("   Add to PATH: export PATH=\"$PATH:$(pwd)/target/release\"")
        sys.exit(1)


