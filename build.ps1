function Cmd([string] $What, [scriptblock] $Action) {
  $LASTEXITCODE = 0

  Write-Host "--- $What ---" -ForegroundColor Green
  & $Action

  if ($LASTEXITCODE -ne 0) {
    throw "$What failed with exit code $LASTEXITCODE."
  }
}

<# Delete a file or directory and all its conents #>
function Remove-Dir([string] $Path) {
  if (Test-Path $Path) {
    Remove-Item $Path -Force -Recurse
  }
}

# Variables
$root = $PSScriptRoot
$distPath = "$root\dist"
$stagingRoot = "$root\staging"
$executableName = "fersk"

# Get full version string
$version = (Get-Content "version.json" | ConvertFrom-Json).FullVersion

function BuildTarget([string] $suffix, [string] $Target) {
  $stagingPath = Join-Path $stagingRoot $Target

  # Create staging directory
  New-Item $stagingPath -ItemType Directory -Force

  Cmd "Build $Target" { cargo build --release --target $Target }
  Copy-Item "target\$Target\release\$executableName.exe" -Destination $stagingPath

  # Create distribution archive
  CreateArchive "$executableName-$version-$suffix" $stagingPath
}

<# Create distribution archive #>
function CreateArchive([string] $ArchiveName, [string] $Path) {
  Push-Location $Path
  try {
    $archivePath = "$distPath\$ArchiveName.zip"

    # Compress archive
    Cmd "Compressing archive" { 7z a -mx9 $archivePath * }

    # Generate checksum for archive
    $checksum = (Get-FileHash -Path $archivePath -Algorithm SHA256).Hash.ToString()
    Set-Content -Path "$archivePath.sha256.txt" -Encoding utf8NoBOM -Value $checksum -NoNewline
  } finally {
    Pop-Location
  }
}

try {
  # Delete old output directories
  Cmd "Clean" {
    Remove-Dir $distPath
    Remove-Dir $stagingRoot
  }

  # Build and stage
  BuildTarget "windows-i686" "i686-pc-windows-msvc"
  BuildTarget "windows-x86_64" "x86_64-pc-windows-msvc"

  # Create output directories
  New-Item $distPath -ItemType Directory -Force

  # Write version file
  Set-Content -Path "$distPath\VERSION.txt" -Encoding utf8NoBOM -Value $version -NoNewline

  # Build Chocolatey package
  Cmd "Build Chocolatey package" { choco pack "chocolatey\package.nuspec" --outputdirectory $distPath --version=$version }

  # Remove version file after building Chocolatey package
  Remove-Item -Path "$distPath\VERSION.txt"
}
catch {
  Write-Host $_.Exception.Message -ForegroundColor Red
}
