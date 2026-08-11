$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
  Write-Host "Running pnpm dependency audit..."
  pnpm --filter quickpick-desktop audit --prod
  if ($LASTEXITCODE -ne 0) {
    throw "pnpm audit failed"
  }

  $cargoAudit = Get-Command cargo-audit -ErrorAction SilentlyContinue
  if (-not $cargoAudit) {
    Write-Host "cargo-audit is not installed. Install it with: cargo install cargo-audit"
    exit 1
  }

  Write-Host "Running cargo dependency audit..."
  Push-Location "apps/desktop/src-tauri"
  try {
    cargo audit
    if ($LASTEXITCODE -ne 0) {
      throw "cargo audit failed"
    }
  } finally {
    Pop-Location
  }

  Write-Host "Security audit passed."
} finally {
  Pop-Location
}
