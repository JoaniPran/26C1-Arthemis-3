# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

All `cargo` commands must be run from `DARG/` (the workspace root).

```bash
cd DARG

# Build
cargo build
cargo build --release

# Run components
cargo run -p ui                                    # Starts UI + embedded coordinator
cargo run -p coordinator                           # Standalone coordinator (port 8080)
cargo run -p coordinator -- 9090                  # Custom port
cargo run -p worker -- <name>                     # Worker connecting to 127.0.0.1:8080
cargo run -p worker -- <name> <ip>               # Worker connecting to remote coordinator

# Tests
cargo test                                         # All tests (unit + integration)
cargo test -p coordinator                         # Single crate
cargo test test_ready_tasks_flow                  # Single test by name
cargo clippy
cargo fmt
```

## Architecture

The project is a distributed DAG workflow orchestrator split into 4 crates:

- **`common`** — Shared `Message` enum (the entire TCP protocol). Any change here affects both coordinator and worker.
- **`coordinator`** — Core server logic + SQLite persistence + artifact HTTP server.
- **`worker`** — Connects to coordinator, executes shell commands, streams output back.
- **`ui`** — egui/eframe GUI; embeds the coordinator by calling `coordinator::server::start_server()` in a background thread.

### Communication Flow

All TCP messages are JSON-serialized `Message` variants, newline-delimited:

```
Worker → Coordinator:  RegisterWorker, Heartbeat, TaskStatus, LogFragment
Coordinator → Worker:  AssignTask
```

The coordinator exposes two ports:
- `:8080` — TCP socket for worker connections (one OS thread per worker in `handler.rs`)
- `:8081` — HTTP via `rouille` for artifact file transfer (`GET /download/<file>`, `POST /upload/<file>`)

### Coordinator Internal Threads

`server::start_server()` spawns:
1. **Watcher thread** (`parser.rs`) — polls `./workflows/` every 2s for new/modified YAML files, validates DAGs with `petgraph`, inserts into SQLite.
2. **Dispatcher thread** (`server.rs`) — polls DB every 2s for `PENDING` tasks whose dependencies are all `SUCCESS`, assigns to a free worker via `CoordinatorState`.
3. **Supervisor/watchdog** (`supervisor.rs`) — checks every 5s for workers with `last_seen > 15s`, evicts them, resets their in-flight tasks to `PENDING`.
4. **Log monitor** (`monitor.rs`) — receives `LogEvent` via mpsc channel, writes to stdout.
5. **Artifact server** — HTTP file server for inter-task artifact passing.
6. **Per-worker thread** — `WorkerHandler::handle_connection()` handles one worker's entire lifecycle.

### Task State Machine

```
SLEEPING → PENDING → RUNNING → SUCCESS
                    └──────→ FAILED
```

Transitions are enforced in `db/tasks.rs`. On coordinator restart, all `RUNNING` tasks are reset to `PENDING` (crash recovery).

### UI ↔ Coordinator IPC

When the UI launches the coordinator, it passes a `mpsc::Sender<String>` (`ui_tx`). The coordinator pushes string events:

| Message format | Meaning |
|---|---|
| `LOADED:<file>:<display_name>` | Workflow successfully parsed and stored |
| `ERROR:<file>:<reason>` | Workflow rejected (cycle, bad YAML, etc.) |
| `STATUS:<task_id>:<STATUS>` | Task status changed |
| `LOG:<task_id>:<line>` | New log line from a task |

The UI also reads `arthemis.db` directly via `rusqlite` for initial state on startup and on workflow selection.

### Worker Execution

Each task runs in an isolated workspace at `/tmp/arthemis_worker/task_<id>/`. The worker:
1. Downloads required artifact ZIPs from the coordinator's HTTP server.
2. Runs the shell command via `std::process::Command`, streaming stdout/stderr as `LogFragment` messages.
3. On success, if `produces` is set, zips the directory and uploads to the coordinator.
4. Sends `TaskStatus { status: "Success" | "Failed" }`.
5. Auto-reconnects on disconnect (5s retry loop in `client.rs`).

### Workflow YAML Format

```yaml
name: "My Pipeline"
tasks:
  - name: "task_a"
    command: "echo hello"
    depends_on: []
  - name: "task_b"
    command: "echo world"
    depends_on: ["task_a"]
    produces: "output_dir"      # optional: directory to zip and upload as artifact
  - name: "task_c"
    command: "process.sh"
    depends_on: ["task_b"]
    consumes: ["task_b"]        # optional: download artifact from task_b before running
```

Workflows are placed in `./workflows/` (relative to where the coordinator/UI process runs). The watcher hot-reloads modified files.

### SQLite Schema

Tables: `workflows`, `tasks`, `dependencies`, `task_consumes`, `task_logs`. All use `ON DELETE CASCADE`. The DB file is `arthemis.db` in the process working directory. Use `:memory:` in tests.

### Concurrency Model

No `async`/`await`. All concurrency uses `std::thread` + `std::sync::mpsc` channels + `Arc<Mutex<T>>`. `CoordinatorState` (`state.rs`) is the in-memory worker registry, shared via `Arc<Mutex<HashMap>>`. `SharedDatabase = Arc<Mutex<Database>>` is the DB handle shared across threads.
