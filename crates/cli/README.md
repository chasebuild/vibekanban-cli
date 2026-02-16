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

## List projects

```bash
vibe-kanban-cli projects
```

JSON output:

```bash
vibe-kanban-cli projects --json
```

## Add project from current folder

```bash
vibe-kanban-cli project add .
```

Override the project name:

```bash
vibe-kanban-cli project add . --name "My Project"
```

## Start the server (background)

```bash
vibe-kanban-cli server start --background
```

If you need a custom command, pass `--command`:

```bash
vibe-kanban-cli server start --background --command "cargo run --bin server"
```
