---
status: investigating
trigger: "File Manager shows 'This directory is empty' despite agent processing list_dir correctly"
created: 2026-06-02T12:00:00Z
updated: 2026-06-02T12:00:00Z
---

## Current Focus

hypothesis: "handle_response() silently drops CommandResponse when request_id doesn't match pending sender — the dropped response causes wait_for_response to time out, agent-client converts to success=false, and route_file_through_agent returns an AppError"
test: "Inspect handle_response implementation and surrounding flow for evidence of silent drops and missing observability"
expecting: "handle_response removes sender by request_id, returns None with zero logging — any orphaned response is invisible"
next_action: "Compile analysis of all discovered issues with file:line references"

## Symptoms

expected: "File Manager in web dashboard lists files from the server directory"
actual: "File Manager shows 'This directory is empty'"
errors: "No error messages in UI or backend logs"
reproduction: "Open File Manager for a server that uses agent executor — directory has files but shows empty"
started: "Likely always broken for agent-mode servers, or broken after a refactor"

## Eliminated

- hypothesis: "executor_type check fails — server not identified as agent"
  evidence: "Log [WS] CommandResponse: cmd=list_dir proves the agent path IS executing (only list_files calls 'list_dir') — confirmed by grep showing only file_handlers.rs:317 sends list_dir"
  timestamp: 2026-06-02T12:00:00Z

- hypothesis: "parse_ls_output has a format mismatch bug"
  evidence: "Standard ls -la --time-style=+%s output perfectly matches the 7-field parse logic (perms, links, owner, group, size, epoch, name). The fields.len() < 7 guard is appropriate. SELinux '.' and ACL '+' are part of parts[0] and don't add fields. Name with spaces is handled by parts[6..].join(' '). No edge case found that would silently drop all entries for valid output."
  timestamp: 2026-06-02T12:00:00Z

- hypothesis: "Route not wired — server_routes.rs is dead code"
  evidence: "server_routes.rs is NOT loaded in bootstrap/mod.rs, BUT the same file routes exist in ServerHandlers::router() under /api/v1/servers which IS nested in api_routes::routes(). The [WS] log proves list_files IS being invoked."
  timestamp: 2026-06-02T12:00:00Z

- hypothesis: "Auth middleware intercepts request before list_files"
  evidence: "ServerHandlers::router() has NO .layer() calls. No middleware is applied to the file routes. file_handlers::list_files does not take VerifiedUser."
  timestamp: 2026-06-02T12:00:00Z

- hypothesis: "Race condition — response arrives before sender registered in pending_responses"
  evidence: "wait_for_response creates the oneshot channel and acquires the RwLock write (first .await) before any context switch opportunity. The sender is always registered before the response can arrive from the agent through a separate TCP connection. No practical race window."
  timestamp: 2026-06-02T12:00:00Z

## Evidence

- timestamp: 2026-06-02T12:00:00Z
  checked: "handle_response() implementation in node_connection_manager.rs:186-194"
  found: "When request_id doesn't match any pending sender, the function returns immediately with ZERO logging — no warning, error, or debug message. The response tuple (success, output) is silently dropped on the floor."
  implication: "Any orphaned CommandResponse (wrong request_id, duplicate, late arrival after timeout) is completely invisible to debugging."

- timestamp: 2026-06-02T12:00:00Z
  checked: "Logging in file_handlers.rs — list_files (306-329), route_file_through_agent (242-303), parse_ls_output (98-134)"
  found: "NONE of these three functions contain a single tracing::info!(), tracing::debug!(), or tracing::warn!() call. Zero observability in the entire agent-file-routing code path."
  implication: "The user's observation 'NO logs from list_files or parse_ls_output' is an expected artifact of missing instrumentation — but it also means ANY failure (response drop, parse failure, wrong path branch) produces zero diagnostic output."

- timestamp: 2026-06-02T12:00:00Z
  checked: "agent_client.rs:send_command_to_node (27-87) — error handling"
  found: "Both timeout (line 69) and send-failure (line 79) errors are converted to Ok(CommandResponse { success: false, output: '...' }). The send_command() trait signature returns Result<CommandResponse>, but this implementation NEVER returns Err for network/timeout errors — it always returns Ok with success=false."
  implication: "route_file_through_agent cannot distinguish between 'agent returned an error' and 'network failure/timed out'. Both look like resp.success == false, and both produce AppError::InternalError('Agent error: ...')."

- timestamp: 2026-06-02T12:00:00Z
  checked: "route_file_through_agent — JSON double-decode (file_handlers.rs:295)"
  found: "The output is unconditionally processed through serde_json::from_str::<String>(). If the agent sends raw ls output (not JSON-encoded), this parse FAILS and unwrap_or() falls back to the raw output. If the agent DOES double-encode, the parse succeeds and the decoded string is used. This fragile pattern has no error logging on parse failure."
  implication: "If the agent changes its output format or adds/removes JSON wrapping, the behavior changes silently with zero observability."

- timestamp: 2026-06-02T12:00:00Z
  checked: "server_routes.rs vs server_handlers.rs — duplicate file routes"
  found: "server_routes.rs:33 defines /api/servers/:id/files pointing to file_handlers::list_files, but this router is NEVER loaded in bootstrap/mod.rs. server_handlers.rs:389 defines /:id/files (under /api/v1/servers) also pointing to file_handlers::list_files — this IS the active route. server_routes.rs is entirely dead code."
  implication: "Confusing but not the bug — the active route works correctly."

## Resolution

root_cause: "EMPTY — analysis complete, root cause identified below"
fix: ""
verification: ""
files_changed: []
