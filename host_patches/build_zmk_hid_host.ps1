param(
    [string]$QmkHostRoot = (Join-Path $env:USERPROFILE "dev\qmk-hid-host"),
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA "zmk-hid-host"),
    [string]$MinGWRoot = (Join-Path $env:USERPROFILE "dev\toolchains\mingw64")
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$patchScript = Join-Path $PSScriptRoot "apply_qmk_hid_host_patches.ps1"
$sourceUrl = "https://github.com/zzeneg/qmk-hid-host"
$sourceRevision = "fceaea1c81887fcb869662c7a62b065083166273"
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
$rustup = Join-Path $env:USERPROFILE ".cargo\bin\rustup.exe"
$mingwBin = Join-Path $MinGWRoot "bin"

if (-not (Test-Path $cargo) -or -not (Test-Path $rustup)) {
    throw "Rust is missing. Install it with rustup before building."
}

if (-not (Test-Path (Join-Path $mingwBin "gcc.exe"))) {
    throw "MinGW was not found at $MinGWRoot. Install MinGW-w64 or pass -MinGWRoot."
}

$env:PATH = "$mingwBin;$env:USERPROFILE\.cargo\bin;$env:PATH"

if (-not (Test-Path (Join-Path $QmkHostRoot ".git"))) {
    New-Item -ItemType Directory -Path (Split-Path -Parent $QmkHostRoot) -Force | Out-Null
    git clone --no-checkout $sourceUrl $QmkHostRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to clone qmk-hid-host."
    }

    git -C $QmkHostRoot checkout --detach $sourceRevision
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to check out the pinned qmk-hid-host revision."
    }
} else {
    $currentRevision = (git -C $QmkHostRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $currentRevision -ne $sourceRevision) {
        throw "Expected qmk-hid-host revision $sourceRevision at $QmkHostRoot; found $currentRevision. No files were changed."
    }
}

& $patchScript -QmkHostRoot $QmkHostRoot
if ($LASTEXITCODE -ne 0) {
    throw "Failed to apply qmk-hid-host patches."
}

& $rustup toolchain list | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Could not query installed Rust toolchains."
}

& $cargo +stable-x86_64-pc-windows-gnu build --release --features silent --target x86_64-pc-windows-gnu --manifest-path (Join-Path $QmkHostRoot "Cargo.toml")
if ($LASTEXITCODE -ne 0) {
    throw "qmk-hid-host build failed."
}

$builtExe = Join-Path $QmkHostRoot "target\x86_64-pc-windows-gnu\release\qmk-hid-host.exe"
if (-not (Test-Path $builtExe)) {
    throw "Build completed but the expected executable was not found: $builtExe"
}

New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null
$runningProcess = Get-Process -Name "zmk-hid-host.silent" -ErrorAction SilentlyContinue
if ($runningProcess) {
    Stop-Process -Id $runningProcess.Id -Force
    $runningProcess.WaitForExit()
}

Copy-Item $builtExe (Join-Path $InstallRoot "zmk-hid-host.silent.exe") -Force
Write-Host "Installed zmk-hid-host.silent.exe to $InstallRoot"