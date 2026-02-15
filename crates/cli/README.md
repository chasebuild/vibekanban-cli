# Vibe Kanban CLI

Interactive CLI for Vibe Kanban.

## Install

```bash
cargo install vibe-kanban-cli
```

## Usage

```bash
vibe-kanban-cli --help
```

## Start the server (background)

```bash
vibe-kanban-cli server start --background
```

If you need a custom command, pass `--command`:

```bash
vibe-kanban-cli server start --background --command "cargo run --bin server"
```
