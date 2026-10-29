param([string]$Executable="$PSScriptRoot\..\dist\ByteForge-Rust.exe",[string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(-not $OutputDirectory){$OutputDirectory=Join-Path (Split-Path $PSScriptRoot -Parent) ('output\qa-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))}
$qa=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path $qa){throw 'Use a new QA directory.'}
New-Item -ItemType Directory -Path $qa|Out-Null
$savedPath=$env:PATH
try{$env:PATH=[Environment]::SystemDirectory+';'+$env:windir;$p=Start-Process -FilePath (Resolve-Path $Executable).Path -ArgumentList @('--hidden','--crash-fixture','--ui-self-test',('"'+$qa+'"')) -WindowStyle Hidden -PassThru}finally{$env:PATH=$savedPath}
try{
    $deadline=[DateTime]::UtcNow.AddSeconds(45)
    while(-not(Test-Path (Join-Path $qa 'result.txt')) -and [DateTime]::UtcNow -lt $deadline -and -not $p.HasExited){Start-Sleep -Milliseconds 100}
    $result=Get-Content -Raw -LiteralPath (Join-Path $qa 'result.txt')
    if(Test-Path (Join-Path $qa 'checks.txt')){Get-Content (Join-Path $qa 'checks.txt')}
    if(-not $result.StartsWith('PASS:')){throw $result}
    $result.Trim()
    # No WM_CLOSE/exit handler: terminate after autosave, then open the same session in a new process.
    Stop-Process -Id $p.Id -Force
    $p.WaitForExit()
    $verify=Join-Path $qa 'restart'
    $savedPath=$env:PATH
    try{$env:PATH=[Environment]::SystemDirectory+';'+$env:windir;$second=Start-Process -FilePath (Resolve-Path $Executable).Path -ArgumentList @('--hidden','--session-dir',('"'+(Join-Path $qa 'session')+'"'),'--verify-recovery',('"'+$verify+'"')) -WindowStyle Hidden -PassThru}finally{$env:PATH=$savedPath}
    try{
        if(-not $second.WaitForExit(30000)){throw 'Recovery restart test timed out'}
        $restored=Get-Content -Raw -LiteralPath (Join-Path $verify 'result.txt')
        if(-not $restored.StartsWith('PASS:')){throw $restored}
        $restored.Trim()
    }finally{if(-not $second.HasExited){Stop-Process -Id $second.Id};$second.Dispose()}
    "Evidence: $qa"
}finally{if(-not $p.HasExited){Stop-Process -Id $p.Id};$p.Dispose()}
