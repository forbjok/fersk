$toolsPath = Split-Path -Parent $MyInvocation.MyCommand.Definition

$packageName = 'fersk'
$version = Get-Content -Path "$toolsPath\VERSION.txt" -Encoding utf8
$url = "https://github.com/forbjok/fersk/releases/download/v$version/fersk-$version-windows-i686.zip"
$url64 = "https://github.com/forbjok/fersk/releases/download/v$version/fersk-$version-windows-x86_64.zip"
$checksum = Get-Content -Path "$toolsPath\fersk-$version-windows-i686.zip.sha256.txt" -Encoding utf8
$checksum64 = Get-Content -Path "$toolsPath\fersk-$version-windows-x86_64.zip.sha256.txt" -Encoding utf8
$unzipLocation = $toolsPath

$packageArgs = @{
  packageName = $packageName
  url = $url
  url64bit = $url64
  checksum = $checksum
  checksumType = "SHA256"
  checksum64 = $checksum64
  checksum64Type = "SHA256"
  unzipLocation = $unzipLocation
}

Install-ChocolateyZipPackage @packageArgs
