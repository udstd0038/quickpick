# Contributing to QuickPick

Thanks for helping improve QuickPick. Contributions, bug reports, translations, and design feedback are welcome.

## Before You Start

- Read `AGENT.md` and the relevant documents under `docs/`.
- Check existing issues and pull requests before opening a new one.
- Keep changes focused on one bug, feature, or documentation improvement.
- Do not include API keys, selected text, screenshots, clipboard contents, or complete AI responses in code, logs, issues, pull requests, or CI artifacts.

## Development Setup

Requirements:

- Node.js 22 or later
- pnpm; use the version declared by `packageManager` in `package.json`
- Rust stable toolchain
- Platform build dependencies required by Tauri 2

```bash
git clone https://github.com/udstd0038/quickpick.git
cd quickpick
pnpm install
pnpm desktop:dev
```

## Validation

Run the checks relevant to your change:

```bash
pnpm test
pnpm build
cd apps/desktop/src-tauri
cargo test
cargo check
```

For full local verification, build the release executable:

```powershell
pnpm --filter quickpick-desktop exec tauri build --no-bundle --ci
```

## Pull Requests

- Use a clear commit message describing the user-visible change.
- Explain what changed, why it changed, and how it was verified.
- Include screenshots for visible UI changes.
- Update `docs/` or `dev-logs/` when behavior, architecture, security, privacy, or accepted UI rules change.
- Keep generated build outputs and local machine files out of commits.

## Security and Privacy

QuickPick is local-first. New features must not add content history or transmit user content without an explicit user action. Report security issues according to `SECURITY.md`, not in a public issue.
