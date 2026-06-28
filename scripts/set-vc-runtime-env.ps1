$crt = Get-ChildItem -Path "$env:ProgramFiles\Microsoft Visual Studio\*\*\VC\Redist\MSVC\*\x64\Microsoft.VC*.CRT" -Directory -ErrorAction SilentlyContinue |
  Sort-Object { try { [version]$_.Parent.Parent.Name } catch { '0.0' } } -Descending |
  Select-Object -First 1

if (-not $crt) {
  Write-Error 'VC runtime CRT directory not found'
  exit 1
}

Write-Host "Using VC runtime from $($crt.FullName)"
Add-Content -Path $env:GITHUB_ENV -Value "DROPSLIM_VC_RUNTIME_DIR=$($crt.FullName)"
