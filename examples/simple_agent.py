#!/usr/bin/env python3
"""
Simple AI Agent using Nexus

This is a basic agent that demonstrates how to:
1. Maintain conversation memory
2. Use tools to interact with the world
3. Process user requests

Run:
    export PATH="$PATH:$(pwd)/target/release"
    python3 examples/simple_agent.py
"""
import sys
import json
import re
from datetime import datetime
from typing import Optional, Dict, Any, List

sys.path.insert(0, "sdk/python")
from nexus_client import NexusClient, NexusError


class SimpleAgent:
    """A basic agent that uses Nexus tools to accomplish tasks."""
    
    def __init__(self):
        self.client = NexusClient()
        self.name = "NexusAgent"
        self.session_id = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Load agent state from memory
        self._load_state()
    
    def _load_state(self):
        """Load agent state from persistent memory."""
        self.interaction_count = int(
            self.client.memory_recall("agent:interaction_count") or "0"
        )
        self.last_user = self.client.memory_recall("agent:last_user")
    
    def _save_state(self):
        """Save agent state to persistent memory."""
        self.client.memory_store("agent:interaction_count", str(self.interaction_count))
        self.client.memory_store("agent:last_session", self.session_id)
    
    def greet(self, user_name: Optional[str] = None) -> str:
        """Generate a greeting based on context."""
        self.interaction_count += 1
        
        if user_name:
            self.client.memory_store("agent:last_user", user_name)
            self.last_user = user_name
        
        # Get current time for context
        time_data = json.loads(self.client.call_text("get_time", {}))
        hour = datetime.fromisoformat(time_data["time"].replace("+00:00", "")).hour
        
        if hour < 12:
            greeting = "Good morning"
        elif hour < 17:
            greeting = "Good afternoon"
        else:
            greeting = "Good evening"
        
        name = user_name or self.last_user or "there"
        
        if self.interaction_count == 1:
            response = f"{greeting}, {name}! I'm {self.name}, your AI assistant."
        else:
            response = f"{greeting}, {name}! Nice to see you again (interaction #{self.interaction_count})."
        
        self._save_state()
        return response
    
    def remember(self, key: str, value: str) -> str:
        """Remember something for the user."""
        mem_key = f"user:{key}"
        self.client.memory_store(mem_key, value)
        return f"Got it! I'll remember that {key} = '{value}'"
    
    def recall(self, key: str) -> str:
        """Recall something the user asked to remember."""
        mem_key = f"user:{key}"
        value = self.client.memory_recall(mem_key)
        if value:
            return f"I remember! {key} = '{value}'"
        else:
            return f"Sorry, I don't have anything stored for '{key}'"
    
    def list_memories(self) -> str:
        """List all user memories."""
        all_keys = self.client.memory_list()
        user_keys = [k.replace("user:", "") for k in all_keys if k.startswith("user:")]
        
        if user_keys:
            return f"I remember these things: {', '.join(user_keys)}"
        else:
            return "I don't have any memories stored yet."
    
    def get_time(self) -> str:
        """Get the current time."""
        time_data = json.loads(self.client.call_text("get_time", {}))
        return f"The current time is {time_data['time']}"
    
    def echo(self, message: str) -> str:
        """Echo a message (for testing)."""
        result = self.client.call_text("echo", {"text": message})
        return f"Echo: {result}"
    
    def process_command(self, command: str) -> str:
        """Process a natural language command."""
        cmd = command.lower().strip()
        
        # Greeting patterns
        if any(g in cmd for g in ["hello", "hi", "hey", "greet"]):
            # Extract name if provided
            match = re.search(r"(?:i'm|i am|my name is|call me)\s+(\w+)", cmd, re.I)
            name = match.group(1) if match else None
            return self.greet(name)
        
        # Remember patterns
        match = re.search(r"remember\s+(?:that\s+)?(\w+)\s+(?:is|=)\s+(.+)", cmd, re.I)
        if match:
            return self.remember(match.group(1), match.group(2))
        
        # Recall patterns
        match = re.search(r"(?:what is|recall|what's)\s+(?:my\s+)?(\w+)", cmd, re.I)
        if match:
            return self.recall(match.group(1))
        
        # List memories
        if any(w in cmd for w in ["memories", "what do you remember", "list"]):
            return self.list_memories()
        
        # Time
        if any(w in cmd for w in ["time", "what time", "current time"]):
            return self.get_time()
        
        # Echo
        if cmd.startswith("echo "):
            return self.echo(command[5:])
        
        # Help
        if "help" in cmd:
            return self.get_help()
        
        # Unknown
        return f"I'm not sure how to handle that. Try 'help' for available commands."
    
    def get_help(self) -> str:
        """Return help text."""
        return """
Available commands:
  • hello / hi - Greet the agent
  • hello, I'm [name] - Greet with your name
  • remember [key] is [value] - Store something in memory
  • what is [key] - Recall from memory
  • list memories - Show all stored memories
  • what time is it - Get current time
  • echo [text] - Echo text back
  • help - Show this help
  • quit / exit - Exit the agent
"""
    
    def run_interactive(self):
        """Run an interactive session."""
        print("=" * 50)
        print(f"  {self.name} - Interactive Mode")
        print("=" * 50)
        print(self.greet())
        print("\nType 'help' for available commands, 'quit' to exit.\n")
        
        while True:
            try:
                user_input = input("You: ").strip()
                
                if not user_input:
                    continue
                
                if user_input.lower() in ["quit", "exit", "bye"]:
                    print(f"\n{self.name}: Goodbye! See you next time.")
                    self._save_state()
                    break
                
                response = self.process_command(user_input)
                print(f"\n{self.name}: {response}\n")
                
            except KeyboardInterrupt:
                print(f"\n\n{self.name}: Goodbye!")
                self._save_state()
                break
            except NexusError as e:
                print(f"\n{self.name}: Oops, something went wrong: {e}\n")


def run_demo():
    """Run a non-interactive demo."""
    print("=" * 50)
    print("  Simple Agent Demo (non-interactive)")
    print("=" * 50)
    
    agent = SimpleAgent()
    
    # Demo commands
    commands = [
        "Hello, I'm Alice",
        "remember favorite_color is blue",
        "remember city is San Francisco",
        "what is favorite_color",
        "list memories",
        "what time is it",
        "echo Testing the echo tool!",
    ]
    
    print()
    for cmd in commands:
        print(f"You: {cmd}")
        response = agent.process_command(cmd)
        print(f"Agent: {response}")
        print()
    
    print("=" * 50)
    print("Demo complete!")
    print("=" * 50)


def main():
    """Main entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == "--demo":
        run_demo()
    else:
        agent = SimpleAgent()
        agent.run_interactive()


if __name__ == "__main__":
    try:
        main()
    except FileNotFoundError:
        print("❌ Error: 'nexus' binary not found.")
        print("   Build: cargo build --release")
        print("   Add to PATH: export PATH=\"$PATH:$(pwd)/target/release\"")
        sys.exit(1)


