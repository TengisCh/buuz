# SPDX-License-Identifier: MIT OR Apache-2.0
# Build the ARM64X forwarder and stage the files for the ARM64 installer.
# Run on Windows ARM64 after both DLLs are built:
#   pwsh windows/arm64x/build.ps1 -Arm64Dll <native dll> -X64Dll <x64 dll> -Settings <exe> -Out <dir>
param(
    [Parameter(Mandatory)] [string] $Arm64Dll,
    [Parameter(Mandatory)] [string] $X64Dll,
    [Parameter(Mandatory)] [string] $Settings,
    [Parameter(Mandatory)] [string] $Out
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
$root = & $vswhere -latest -products * -property installationPath
if (-not $root) { throw "Visual Studio not found" }
# The linker that runs on this machine and targets ARM64.
$link = Get-ChildItem (Join-Path $root "VC\Tools\MSVC") -Recurse -Filter link.exe |
    Where-Object { $_.FullName -match "Hostarm64\\arm64\\" } | Select-Object -Last 1
if (-not $link) { throw "ARM64 link.exe not found" }

New-Item -ItemType Directory -Force $Out | Out-Null
Copy-Item $Arm64Dll (Join-Path $Out "buuz_tsf_arm64.dll")
Copy-Item $X64Dll (Join-Path $Out "buuz_tsf_x64.dll")
Copy-Item $Settings (Join-Path $Out "buuz_settings.exe")
# The word list sits next to the DLL: the build copies it beside the ARM64 DLL.
Copy-Item (Join-Path (Split-Path -Parent $Arm64Dll) "mn.txt") (Join-Path $Out "mn.txt")

# A pure forwarder has no code: only two export tables, one for each half.
& $link.FullName /nologo /dll /noentry /nodefaultlib /ignore:4001 /machine:arm64x `
    "/defArm64Native:$here\forward_arm64.def" "/def:$here\forward_x64.def" `
    "/out:$Out\buuz_tsf.dll"
if ($LASTEXITCODE -ne 0) { throw "link /machine:arm64x failed: $LASTEXITCODE" }
Get-ChildItem $Out
