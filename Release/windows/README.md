# Windows Release Build — Fount

Builds MSIX package + portable executable for Windows Store submission.

## Prerequisites

- **Rust** — `rustup` + `x86_64-pc-windows-msvc` target
- **Windows SDK** — comes with Visual Studio Build Tools or Visual Studio
- **ImageMagick** — `winget install ImageMagick` (for generating MSIX assets)
- **Git** — for version info

## Quick Start

```powershell
.\Release\windows\build-msix.ps1 -SelfSign
```

This will:
1. Build the Rust binary (`cargo build --release`)
2. Create an MSIX package signed with a self-signed cert (for local testing)
3. Copy a portable `fount.exe`
4. All output → `Release/artifacts/`

## Signing Options

| Flag | What it does |
|---|---|
| `-SelfSign` | Creates a matching cert on this machine, signs the MSIX. Good for local testing. |
| `-SkipSigning` | Produces unsigned MSIX. The Store signs it during ingestion. |
| `-PfxPath "file.pfx" -PfxPassword "xxx"` | Signs with your real code signing certificate. |

## Examples

```powershell
# Local testing (quick)
.\Release\windows\build-msix.ps1 -SelfSign

# Unsigned for Store upload
.\Release\windows\build-msix.ps1 -SkipSigning

# Signed with real cert
.\Release\windows\build-msix.ps1 -PfxPath "C:\certs\fount.pfx" -PfxPassword $pass
```

## Output

Everything goes to `Release/artifacts/`:

```
Release/artifacts/
  Fount-<version>.msix              → Windows Store upload
  Fount-Portable-x64-<version>.exe  → Standalone binary
```
