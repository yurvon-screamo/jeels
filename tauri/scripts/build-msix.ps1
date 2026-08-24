#requires -Version 5.1
<#
.SYNOPSIS
    Builds the Microsoft Store MSIX package for Origa (single source of truth
    for staging + packaging; CI's _build-windows-store.yml calls this script).

.DESCRIPTION
    Pipeline:
      1. Map -Version to a valid MSIX dotted quad:
           ^\d+\.\d+\.\d+$  -> X.Y.Z.0   (submittable stable channel)
           anything else    -> 0.0.0.<GITHUB_RUN_NUMBER or 1>  (smoke-only;
             never submit these — Store requires strictly increasing versions)
      2. Build the store-flavored binary (ORIGA_APP_STORE=1 compiles out the
         self-update machinery per Store policy 10.2.5) with --no-bundle.
      3. Stage Origa.exe (+ WebView2Loader.dll when present) together with
         msix/Package.appxmanifest and msix/Assets/.
      4. Inject Partner Center identity + version into the staged manifest.
      5. Pack with MakeAppx and sign with an ephemeral self-signed certificate
         whose CN matches the manifest Publisher (Add-AppxPackage requires the
         match). The Microsoft Store re-signs submitted packages with its own
         certificate, so the self-signature exists purely to make the artifact
         locally installable for smoke testing.

    The Apple-only PrivacyInfo.xcprivacy resource is deliberately NOT staged.

.PARAMETER Version
    Upstream version string (e.g. from the version action). Non X.Y.Z values
    fall back to the smoke-version rule above.

.PARAMETER IdentityName, Publisher, PublisherDisplayName
    Partner Center Product Identity values. Keep the INJECT_* placeholders
    until the MSIX product exists there; local smoke installs work fine with
    placeholder values.

.PARAMETER OutputDirectory
    Where stage/, the .msix and the ephemeral .pfx (+ password file) land.

.PARAMETER SkipBuild
    Re-package an existing target/release/origa-app.exe (fast iteration).

.PARAMETER KeepUnsigned
    Pack without signing (contingency if Partner Center upload validation ever
    rejects self-signed packages; such artifacts are not locally installable).

.NOTES
    Requires the Windows SDK tools (MakeAppx.exe, signtool.exe) — present on
    GitHub windows runners and on any machine with the MSVC Rust toolchain.
#>
[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$IdentityName = "INJECT_PARTNER_CENTER_IDENTITY_NAME",
    [string]$Publisher = "INJECT_PARTNER_CENTER_PUBLISHER",
    [string]$PublisherDisplayName = "INJECT_PARTNER_CENTER_PUBLISHER_DISPLAY_NAME",
    [string]$OutputDirectory = "",
    [switch]$SkipBuild,
    [switch]$KeepUnsigned
)

$ErrorActionPreference = "Stop"

$TauriDir = Split-Path -Parent $PSScriptRoot
$RepoRoot = Split-Path -Parent $TauriDir
if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $TauriDir "target\msix"
}

function Find-SdkTool {
    param([Parameter(Mandatory)][string]$Name)
    $kitRoot = "C:\Program Files (x86)\Windows Kits\10\bin"
    if (-not (Test-Path $kitRoot)) { return $null }
    # Newest installed SDK version first.
    $kits = Get-ChildItem $kitRoot -Directory |
        Sort-Object Name -Descending
    foreach ($kit in $kits) {
        $candidate = Join-Path $kit.FullName "x64\$Name"
        if (Test-Path $candidate) { return $candidate }
    }
    return $null
}

# --- 1. Version mapping ------------------------------------------------------

if ($Version -match '^\d+\.\d+\.\d+$') {
    $MsixVersion = "$Version.0"
}
else {
    $runNumber = if ($env:GITHUB_RUN_NUMBER) { $env:GITHUB_RUN_NUMBER } else { "1" }
    $MsixVersion = "0.0.0.$runNumber"
    Write-Host "Version '$Version' is not X.Y.Z; using smoke fallback $MsixVersion"
}

# --- 2. Build ----------------------------------------------------------------

if (-not $SkipBuild) {
    # build.rs: cfg(app_store), strips updater config. Save/restore so a
    # local smoke run does not silently poison the rest of the shell session
    # (subsequent cargo builds would come out store-flavored).
    $hadStoreEnv = Test-Path Env:\ORIGA_APP_STORE
    $prevStoreEnv = if ($hadStoreEnv) { $env:ORIGA_APP_STORE } else { $null }
    $env:ORIGA_APP_STORE = "1"
    Push-Location $TauriDir
    try {
        npx tauri build --no-bundle
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed with exit code $LASTEXITCODE" }
    }
    finally {
        Pop-Location
        if ($hadStoreEnv) { $env:ORIGA_APP_STORE = $prevStoreEnv }
        else { Remove-Item Env:\ORIGA_APP_STORE -ErrorAction SilentlyContinue }
    }
}

$ExePath = Join-Path $TauriDir "target\release\origa-app.exe"
if (-not (Test-Path $ExePath)) {
    throw "Built binary not found at '$ExePath'. Run the build step first."
}

# --- 3. Stage ----------------------------------------------------------------

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$Stage = Join-Path $OutputDirectory "stage"
if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
New-Item -ItemType Directory -Path $Stage | Out-Null

Copy-Item $ExePath (Join-Path $Stage "Origa.exe")
Copy-Item (Join-Path $TauriDir "msix\Package.appxmanifest") $Stage
Copy-Item (Join-Path $TauriDir "msix\Assets") (Join-Path $Stage "Assets") -Recurse

# Some Tauri/wry versions emit WebView2Loader.dll next to the raw cargo
# binary; include it whenever present (absent = statically linked).
$Loader = Join-Path $TauriDir "target\release\WebView2Loader.dll"
if (Test-Path $Loader) {
    Copy-Item $Loader $Stage
    Write-Host "Staged WebView2Loader.dll"
}

# --- 4. Manifest identity + version injection --------------------------------

$ManifestPath = Join-Path $Stage "Package.appxmanifest"
$manifest = Get-Content $ManifestPath -Raw

function ConvertTo-XmlText([string]$Value) {
    # Identity/Publisher values are pasted from Partner Center by hand and
    # may contain XML-hostile characters; unescaped '&'/'<' crash MakeAppx
    # with an opaque parse error.
    [System.Security.SecurityElement]::Escape($Value)
}

foreach ($pair in @(
        @("__PARTNER_CENTER_IDENTITY_NAME__", (ConvertTo-XmlText $IdentityName)),
        @("__PARTNER_CENTER_PUBLISHER__", (ConvertTo-XmlText $Publisher)),
        @("__PARTNER_CENTER_PUBLISHER_DISPLAY_NAME__", (ConvertTo-XmlText $PublisherDisplayName))
    )) {
    if ($manifest -notmatch [regex]::Escape($pair[0])) {
        throw "Placeholder '$($pair[0])' missing from manifest — template drift."
    }
    $manifest = $manifest.Replace($pair[0], $pair[1])
}
$manifest = $manifest.Replace('Version="0.0.0.0"', "Version=`"$MsixVersion`"")
if ($manifest -notmatch ('Version="' + [regex]::Escape($MsixVersion) + '"')) {
    # Silent no-op here would ship a 0.0.0.0 package that Partner Center
    # rejects with a misleading version error — fail loudly instead.
    throw "Version placeholder replacement failed — template drift in Package.appxmanifest."
}
Set-Content -Path $ManifestPath -Value $manifest -Encoding UTF8

# --- 5. Ephemeral self-signed certificate ------------------------------------

$MsixPath = Join-Path $OutputDirectory ("Origa_{0}_x64.msix" -f $MsixVersion)
$pfxPath = Join-Path $OutputDirectory "Origa.msix.pfx"
$passwordFile = Join-Path $OutputDirectory "Origa.msix.pfx.password.txt"

if (-not $KeepUnsigned) {
    # CN must equal the manifest Publisher or Add-AppxPackage rejects the
    # signature. On CI the cert store is clean, so this is ephemeral per
    # run; locally an existing matching cert is reused. The .pfx travels
    # WITH the artifact — without it nobody outside this machine can
    # validate or install the package.
    $pfxPassword = [System.Guid]::NewGuid().ToString("N")
    $securePassword = ConvertTo-SecureString -String $pfxPassword -Force -AsPlainText
    $subject = "CN=$Publisher"
    Write-Host "Looking for a code-signing certificate '$subject' in CurrentUser\My"
    $cert = Get-ChildItem Cert:\CurrentUser\My |
        Where-Object { $_.Subject -eq $subject } |
        Sort-Object NotBefore -Descending |
        Select-Object -First 1
    if (-not $cert) {
        Write-Host "Not found — generating a new self-signed certificate"
        $cert = New-SelfSignedCertificate `
            -Type Custom `
            -Subject $subject `
            -KeyUsage DigitalSignature `
            -FriendlyName "Origa MSIX packaging (ephemeral)" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
    }
    Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $securePassword | Out-Null
    Set-Content -Path $passwordFile -Value $pfxPassword
}

# --- 6. Pack (+ sign) ---------------------------------------------------------

$makeAppx = Find-SdkTool "MakeAppx.exe"
if (-not $makeAppx) { throw "MakeAppx.exe not found — install the Windows SDK." }

& $makeAppx pack /o /d $Stage /p $MsixPath
if ($LASTEXITCODE -ne 0) { throw "MakeAppx pack failed with exit code $LASTEXITCODE" }

if (-not $KeepUnsigned) {
    $signtool = Find-SdkTool "signtool.exe"
    if (-not $signtool) { throw "signtool.exe not found — install the Windows SDK." }
    & $signtool sign `
        /fd SHA256 /td SHA256 `
        /tr http://timestamp.digicert.com `
        /f $pfxPath /p $pfxPassword `
        $MsixPath
    if ($LASTEXITCODE -ne 0) { throw "signtool sign failed with exit code $LASTEXITCODE" }
}

Write-Host ""
Write-Host "=== MSIX build complete ==="
Write-Host "Package : $MsixPath"
if (-not $KeepUnsigned) {
    Write-Host "Cert    : $pfxPath"
    Write-Host "Password: $passwordFile"
    Write-Host "Install : import the pfx into CurrentUser\Trusted People, then:"
    Write-Host "          Add-AppxPackage -Path `"$MsixPath`""
}
