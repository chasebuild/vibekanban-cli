# Repository Guidelines - Repo: /Users/chungquantin/Developer/vibe-kanban

## Project Structure & Module Organization
- Rust workspace crates: `crates/` (`server`, `db`, `executors`, `services`, `utils`, `deployment`, `local-deployment`, `remote`).
- Frontend app: `frontend/` (React + TypeScript, Vite, Tailwind), source in `frontend/src`.
- Dialog components: `frontend/src/components/dialogs`.
- Remote deployment frontend: `remote-frontend/`.
- Generated shared types: `shared/types.ts` (do not edit directly).
- Assets: `assets/`, `dev_assets_seed/`, `dev_assets/`.
- CLI package: `npx-cli/`.
- Dev helpers: `scripts/`.
- Docs: `docs/`.

## Shared Types (Rust -> TypeScript)
- Use `ts-rs` with `#[derive(TS)]` on Rust types.
- Regenerate with `pnpm run generate-types`.
- Edit `crates/server/src/bin/generate_types.rs`, not `shared/types.ts`.

## Build, Test, and Development Commands
- Install: `pnpm i`.
- Dev (frontend + backend, auto ports): `pnpm run dev`.
- Backend watch: `pnpm run backend:dev:watch`.
- Frontend dev: `pnpm run frontend:dev`.
- Type checks: `pnpm run check` (frontend) and `pnpm run backend:check` (Rust cargo check).
- Rust tests: `cargo test --workspace`.
- Generate TS types: `pnpm run generate-types` (or `generate-types:check` in CI).
- Prepare SQLx (offline): `pnpm run prepare-db`.
- Prepare SQLx (remote package, postgres): `pnpm run remote:prepare-db`.
- Local NPX build: `pnpm run build:npx` then `pnpm pack` in `npx-cli/`.

## Automated QA
- Prefer `pnpm run dev:qa` over `pnpm run dev` when testing app changes.

## Coding Style & Naming Conventions
- Rust: `rustfmt` enforced (`rustfmt.toml`); group imports by crate; snake_case modules, PascalCase types.
- TypeScript/React: ESLint + Prettier (2 spaces, single quotes, 80 cols). PascalCase components, camelCase vars/functions, kebab-case file names where practical.
- Keep functions small; add `Debug`/`Serialize`/`Deserialize` where useful.

## Testing Guidelines
- Rust: prefer unit tests alongside code (`#[cfg(test)]`); add tests for new logic and edge cases.
- Frontend: ensure `pnpm run check` and `pnpm run lint` pass; add lightweight tests (e.g., Vitest) for new runtime logic.

## Commit & Pull Request Guidelines
- Keep PRs small and single-purpose; include summary, testing notes, and risk/rollback considerations.
- Use `codex/` as the branch prefix for new branches.
- Favor atomic commits: one logical change per commit with focused diffs and clear, imperative messages.
- Auto-commit only when changes are clearly scoped and verified, or when explicitly requested; otherwise confirm before committing.
- Never amend or force-push unless explicitly requested.

## Security & Configuration Tips
- Use `.env` for local overrides; never commit secrets.
- Key envs: `FRONTEND_PORT`, `BACKEND_PORT`, `HOST`.
- Dev ports and assets are managed by `scripts/setup-dev-environment.js`.
