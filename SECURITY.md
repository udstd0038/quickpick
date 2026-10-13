# Security Policy

## Supported Versions

Security fixes are provided for the latest stable release on the `main` branch. Pre-release builds are supported on a best-effort basis.

## Reporting a Vulnerability

Report vulnerabilities through GitHub private vulnerability reporting:

https://github.com/udstd0038/quickpick/security/advisories/new

Do not open a public issue for a vulnerability that could expose credentials, clipboard contents, selected text, screenshots, local files, or remote code execution.

Please include:

- The affected QuickPick version and operating system.
- Steps required to reproduce the issue.
- The expected and observed behavior.
- A minimal proof of concept when safe to share.
- Redacted logs or screenshots that do not contain sensitive user content.

## Sensitive Data

Never submit real API keys, access tokens, selected text, screenshots, clipboard contents, or complete AI responses. Use synthetic test data and redact credentials before sharing evidence.

## Scope

In scope:

- Local secret storage and API key handling.
- Network request policy and endpoint validation.
- Clipboard and selected-text handling.
- Screenshot capture and temporary in-memory data.
- Tauri permissions, CSP, command boundaries, and dependency vulnerabilities.

Out of scope:

- Vulnerabilities in third-party AI providers.
- Issues caused by intentionally unsafe local HTTP endpoints configured by the user.
- Social engineering or physical access to an unlocked device.
