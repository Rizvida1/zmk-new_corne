param(
    [string]$QmkHostRoot = "C:\Users\RabR\dev\qmk-hid-host"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$patchRoot = Join-Path $repoRoot "host_patches\qmk-hid-host\src"
$targetRoot = Join-Path $QmkHostRoot "src"

if (-not (Test-Path $patchRoot)) {
    throw "Patch source not found: $patchRoot"
}

if (-not (Test-Path $targetRoot)) {
    throw "Target src folder not found: $targetRoot"
}

$files = @(
    "main.rs",
    "data_type.rs",
    "providers\weather.rs",
    "providers\media\windows.rs"
)

foreach ($relPath in $files) {
    $src = Join-Path $patchRoot $relPath
    $dst = Join-Path $targetRoot $relPath
    $dstDir = Split-Path -Parent $dst

    if (-not (Test-Path $src)) {
        throw "Missing patch file: $src"
    }

    if (-not (Test-Path $dstDir)) {
        New-Item -ItemType Directory -Path $dstDir -Force | Out-Null
    }

    Copy-Item -Path $src -Destination $dst -Force
    Write-Host "Patched $relPath"
}

Write-Host "All qmk-hid-host patches applied to $QmkHostRoot"
