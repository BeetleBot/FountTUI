param(
    [string]$PfxPath = "",
    [string]$PfxPassword = "",
    [switch]$SelfSign,
    [switch]$SkipSigning,
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot\..\.."
$ArtifactsDir = "$ProjectRoot\Release\artifacts"

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }

# --- Detect version ---
if (-not $Version) {
    $cargo = Get-Content "$ProjectRoot\Cargo.toml"
    $currentVersion = [regex]::Match($cargo, '^version = "([^"]+)"').Groups[1].Value
    $Version = Read-Host "Enter version number (current is $currentVersion, press Enter to keep)"
    if (-not $Version) { $Version = $currentVersion }
}
$MsixVersion = "$Version.0"
Write-Step "Building Fount v$Version (MSIX)"

# --- Step 1: Build Rust binary ---
Write-Step "Building Rust binary (release)"
Push-Location $ProjectRoot
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }
Pop-Location

# --- Step 2: Prepare MSIX layout ---
Write-Step "Preparing MSIX layout"
$Layout = "$ProjectRoot\target\msix\layout"
$AssetsDir = "$Layout\Assets"
if (Test-Path $Layout) { Remove-Item $Layout -Recurse -Force }
New-Item -ItemType Directory -Path $AssetsDir -Force | Out-Null

Copy-Item "$ProjectRoot\target\release\fount.exe" $Layout
Copy-Item "$ProjectRoot\packaging\msix\AppxManifest.xml" $Layout

# Generate MSIX assets from source icon
$sizes = @{
    "StoreLogo"       = "50x50"
    "Square44x44Logo" = "44x44"
    "Square150x150Logo" = "150x150"
    "Wide310x150Logo" = "310x150"
    "LargeSquareLogo" = "310x310"
}
foreach ($entry in $sizes.GetEnumerator()) {
    magick "$ProjectRoot\assets\icons\FountTUI_Logo.png" `
        -resize $entry.Value `
        -gravity center `
        -background "#1a1b26" `
        -extent $entry.Value `
        "$AssetsDir\$($entry.Key).png"
}

# --- Step 3: Stamp version into manifest ---
Write-Step "Stamping version $MsixVersion into manifest"
$xml = [xml](Get-Content "$Layout\AppxManifest.xml")
$xml.Package.Identity.Version = $MsixVersion
$xml.Save("$Layout\AppxManifest.xml")

# --- Step 4: Create MSIX with MakeAppx ---
Write-Step "Creating MSIX package"
$SdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
$SdkDirs = Get-ChildItem "$SdkRoot\10.*" -Directory | Sort-Object Name -Descending
if (-not $SdkDirs) { throw "Windows SDK not found at $SdkRoot" }
$SdkBin = Join-Path $SdkDirs[0].FullName "x64"

$MakeAppx = "$SdkBin\MakeAppx.exe"
$MsixFile = "Fount-$Version.msix"
$MsixPath = "$ArtifactsDir\$MsixFile"
New-Item -ItemType Directory -Path $ArtifactsDir -Force | Out-Null
Remove-Item $MsixPath -ErrorAction SilentlyContinue

Write-Host "  MakeAppx: $MakeAppx"
Write-Host "  Output:   $MsixPath"
& $MakeAppx pack /d $Layout /p $MsixPath /o
if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed" }

# --- Step 5: Sign ---
$Signed = $false
if ($SkipSigning) {
    Write-Step "Skipping signing (unsigned - Store will sign)"
} elseif ($SelfSign) {
    Write-Step "Self-signing with matching certificate"
    $Subject = "CN=A5C810D1-3C33-4DED-95DA-33D6BC28A3B0"
    $existing = Get-ChildItem -Path "Cert:\CurrentUser\My" | Where-Object { $_.Subject -eq $Subject }
    if (-not $existing) {
        $cert = New-SelfSignedCertificate -Type Custom `
            -Subject $Subject `
            -KeySpec Signature `
            -KeyExportPolicy Exportable `
            -KeyUsage DigitalSignature `
            -TextExtension "2.5.29.37={text}1.3.6.1.5.5.7.3.3" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -Provider "Microsoft Enhanced RSA and AES Cryptographic Provider" `
            -HashAlgorithm SHA256
        Write-Host "  Created self-signed cert in CurrentUser\My"
        $existing = $cert
    } else {
        $existing = $existing[0]
        Write-Host "  Found existing self-signed cert"
    }

    $IsAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    $machineRootCerts = Get-ChildItem -Path "Cert:\LocalMachine\Root" -ErrorAction SilentlyContinue | Where-Object { $_.Subject -eq $Subject }
    if (-not $machineRootCerts) {
        Write-Step "Installing certificate to LocalMachine stores (admin needed)..."
        $tempCert = "$env:TEMP\fount-selfsign.cer"
        Export-Certificate -Cert $existing -FilePath $tempCert -Type CERT | Out-Null
        if (-not $IsAdmin) {
            $installScript = @"
                `$ErrorActionPreference = 'Stop'
                `$store = New-Object System.Security.Cryptography.X509Certificates.X509Store 'Root','LocalMachine'
                `$store.Open('ReadWrite')
                `$store.Add(New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 '$tempCert')
                `$store.Close()
                `$pub = New-Object System.Security.Cryptography.X509Certificates.X509Store 'TrustedPublisher','LocalMachine'
                `$pub.Open('ReadWrite')
                `$pub.Add(New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 '$tempCert')
                `$pub.Close()
"@
            $encodedScript = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($installScript))
            $proc = Start-Process -Verb RunAs -Wait -PassThru -FilePath "powershell" -ArgumentList "-ExecutionPolicy", "Bypass", "-EncodedCommand", $encodedScript
            if ($proc.ExitCode -ne 0) { Write-Host "  WARNING: Certificate install may have failed (exit $($proc.ExitCode))" -ForegroundColor Yellow }
        } else {
            $store = New-Object System.Security.Cryptography.X509Certificates.X509Store "Root","LocalMachine"
            $store.Open("ReadWrite")
            $store.Add($existing)
            $store.Close()
            $pub = New-Object System.Security.Cryptography.X509Certificates.X509Store "TrustedPublisher","LocalMachine"
            $pub.Open("ReadWrite")
            $pub.Add($existing)
            $pub.Close()
        }
        Remove-Item $tempCert -ErrorAction SilentlyContinue
        Write-Host "  Certificate installed to LocalMachine\Root and TrustedPublisher"
    } else {
        Write-Host "  Certificate already in LocalMachine\Root"
    }

    $Signtool = "$SdkBin\signtool.exe"
    & $Signtool sign /a /fd SHA256 /sha1 $existing.Thumbprint $MsixPath
    if ($LASTEXITCODE -eq 0) { $Signed = $true }
} elseif ($PfxPath) {
    Write-Step "Signing with provided PFX certificate"
    if (-not (Test-Path $PfxPath)) { throw "PFX file not found: $PfxPath" }
    $Signtool = "$SdkBin\signtool.exe"
    $signArgs = @("sign", "/fd", "SHA256", "/f", $PfxPath)
    if ($PfxPassword) { $signArgs += "/p"; $signArgs += $PfxPassword }
    $signArgs += $MsixPath
    & $Signtool $signArgs
    if ($LASTEXITCODE -eq 0) { $Signed = $true }
} else {
    Write-Step "WARNING: No signing option specified. MSIX will be unsigned."
    Write-Host "  Use -SkipSigning, -SelfSign, or -PfxPath/-PfxPassword"
}

if ($Signed) { Write-Host "  Signed successfully" -ForegroundColor Green }

# --- Step 6: Copy portable exe ---
Write-Step "Copying portable executable"
$PortableExe = "Fount-Portable-x64-$Version.exe"
Copy-Item "$ProjectRoot\target\release\fount.exe" "$ArtifactsDir\$PortableExe"

# --- Step 7: Cleanup layout ---
Remove-Item $Layout -Recurse -Force

# --- Done ---
Write-Step "Done! Artifacts in: $ArtifactsDir"
Write-Host "  MSIX:       $MsixFile" -ForegroundColor Green
Write-Host "  Portable:   $PortableExe" -ForegroundColor Green
if (-not $Signed -and -not $SkipSigning -and -not $PfxPath -and -not $SelfSign) {
    Write-Host "  NOTE: MSIX is unsigned. Use -SkipSigning or -SelfSign or -PfxPath" -ForegroundColor Yellow
}
