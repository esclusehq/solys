---
status: awaiting_human_verify
trigger: "On the Nodes page in the System Info section, some fields show 'N/A' instead of actual values: Agent Version, Memory, CPU Cores. Meanwhile OS (linux) and Podman Version (29.3.0) show correctly."
created: 2026-04-22T00:00:00Z
updated: 2026-04-22T00:00:00Z
---

## Current Focus

hypothesis: "Node system info (Agent Version, Memory, CPU Cores) is not being captured/stored properly during node registration or heartbeat"
test: "Trace how node info flows from agent to database to frontend"
expecting: "Find where the data should be stored but isn't"
next_action: "Search for node registration/heartbeat code to understand how system info is sent"

## Symptoms

expected: Agent Version, Memory, CPU Cores should show actual values from the connected node
actual: These fields show "N/A" while OS and Podman Version show correctly
errors: No errors visible in console
reproduction: Navigate to /nodes, click on a node to view System Info
started: Always been N/A since implementation

## Eliminated

- 

## Evidence

- Working fields: OS (linux), Podman Version (29.3.0) - these show correctly
- Non-working fields: Agent Version, Memory, CPU Cores - show "N/A"
- Both sets should come from same source (node registration/heartbeat)

**Evidence Found:**
- timestamp: 2026-04-22
  checked: web-agent/src/agent_connection.rs lines 240-250
  found: "The Register message sends total_memory: None, cpu_cores: None, and is MISSING agent_version field entirely"
  implication: "This is why the fields show N/A - the agent never sends this data!"

- timestamp: 2026-04-22
  checked: api/src/presentation/ws/node_protocol.rs lines 10-24
  found: "Backend expects total_memory, cpu_cores, AND agent_version in Register message"
  implication: "Web-agent is missing agent_version field in its Register struct"

## Resolution

root_cause: "The web-agent never sent total_memory, cpu_cores, or agent_version during node registration - they were hardcoded as None/omitted from the Register message"

fix: "Modified web-agent to collect system info using sysinfo crate and include total_memory, cpu_cores, and agent_version in the Register message sent during WebSocket connection"

verification: "Code compiles successfully with cargo check. After redeploying the web-agent, nodes will send this info during registration and the fields will populate in the System Info section."

files_changed: 
- "web-agent/Cargo.toml: Added sysinfo dependency"
- "web-agent/src/agent_connection.rs: Added sysinfo import, added agent_version field to AgentMessage::Register, added code to collect system metrics during registration"
- "web-agent/temp.rs: Duplicate file with same issue (not part of main codebase)"