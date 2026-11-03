param([string]$Executable="$PSScriptRoot\..\dist\ByteForge-Rust.exe",[string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(-not $OutputDirectory){$OutputDirectory=Join-Path (Split-Path $PSScriptRoot -Parent) ('output\update-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))}
$out=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $out){throw 'Use a new update QA directory.'}
$p=Start-Process -FilePath (Resolve-Path $Executable).Path -ArgumentList @('--hidden','--verify-update',('"'+$out+'"')) -WindowStyle Hidden -PassThru
try {
    if(-not $p.WaitForExit(45000)){throw 'Live GitHub update check timed out.'}
    Get-Content -LiteralPath (Join-Path $out 'update.txt')
    $result=Get-Content -Raw -LiteralPath (Join-Path $out 'result.txt')
    if(-not $result.StartsWith('PASS:')){throw $result}
    $result.Trim()
    "Evidence: $out"
} finally { if(-not $p.HasExited){Stop-Process -Id $p.Id -Force};$p.Dispose() }
