param([string]$Executable="$PSScriptRoot\..\dist\ByteForge-Rust.exe",[string]$OutputDirectory)
$ErrorActionPreference='Stop'
$exe=(Resolve-Path $Executable).Path
if(-not $OutputDirectory){$OutputDirectory=Join-Path (Split-Path $PSScriptRoot -Parent) ('output\explorer-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))}
$root=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $root){throw 'Use a new Explorer QA directory.'}
New-Item -ItemType Directory -Path (Join-Path $root 'input') -Force|Out-Null
$children=[Collections.Generic.List[Diagnostics.Process]]::new()
$checks=[Collections.Generic.List[string]]::new()
function Start-TestProcess([string[]]$Arguments){
    $oldPath=$env:PATH
    try{
        $env:PATH=[Environment]::SystemDirectory+';'+$env:windir
        $p=Start-Process -FilePath $exe -ArgumentList $Arguments -WorkingDirectory (Join-Path $root 'input') -WindowStyle Hidden -PassThru
        $children.Add($p)
        return $p
    }finally{$env:PATH=$oldPath}
}
function Quote([string]$Value){return '"'+$Value+'"'}
function Wait-Result([Diagnostics.Process]$Process,[string]$Folder){
    $file=Join-Path $Folder 'result.txt';$deadline=[DateTime]::UtcNow.AddSeconds(45)
    while(-not(Test-Path -LiteralPath $file) -and -not $Process.HasExited -and [DateTime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 50}
    if(-not(Test-Path -LiteralPath $file)){throw "No test result: $Folder"}
    $result=[IO.File]::ReadAllText($file)
    if(-not $result.StartsWith('PASS:')){throw $result}
    return $result.Trim()
}
function Wait-Count([string]$Session,[int]$Count){
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    while([DateTime]::UtcNow -lt $deadline){
        try{ $b=[IO.File]::ReadAllBytes((Join-Path $Session 'session.bfs'));if($b.Length -gt 16 -and [BitConverter]::ToUInt32($b,8) -eq $Count){return} }catch{}
        Start-Sleep -Milliseconds 100
    }
    throw "Session did not reach $Count tabs: $Session"
}
function Send-Files([string]$Session,[string[]]$Files){
    $p=Start-TestProcess (@('--hidden','--session-dir',(Quote $Session)) + @($Files|ForEach-Object{Quote $_}))
    if(-not $p.WaitForExit(12000)){throw 'A secondary launch did not hand off and exit; possible session-lock popup.'}
    if($p.ExitCode -ne 0){throw "Secondary launch exited with $($p.ExitCode)"}
}
function Verify-Session([string]$Session,[string]$Label,[string[]]$Expected){
    $out=Join-Path $root $Label;New-Item -ItemType Directory -Path $out|Out-Null
    [IO.File]::WriteAllLines((Join-Path $out 'expected.txt'),$Expected,[Text.UTF8Encoding]::new($false))
    $p=Start-TestProcess @('--hidden','--session-dir',(Quote $Session),'--verify-explorer',(Quote $out))
    $result=Wait-Result $p $out
    if(-not $p.WaitForExit(5000)){throw 'Verification process did not exit'}
    $checks.Add("$Label - $result")
}
try{
    $names=@('Incoming café A.json','Incoming B.ps1','Incoming C.ini','Workspace B.txt','Workspace B extra.txt')
    foreach($name in $names){[IO.File]::WriteAllText((Join-Path (Join-Path $root 'input') $name),('fixture: '+$name),[Text.UTF8Encoding]::new($false))}
    $a=Join-Path $root 'primary';$sessionA=Join-Path $a 'session'
    $primary=Start-TestProcess @('--hidden','--crash-fixture','--ui-self-test',(Quote $a))
    $checks.Add(('Baseline - '+(Wait-Result $primary $a)))
    Send-Files $sessionA $names[0..1]
    $checks.Add('One launch hands off multiple relative paths, spaces and Unicode names')
    Send-Files $sessionA @($names[2])
    Send-Files $sessionA @((Join-Path $a 'utf16.txt'))
    Send-Files $sessionA @()
    Wait-Count $sessionA 6
    $checks.Add('Subsequent and bare launches reuse the primary; reopening a path creates no duplicate tab')
    # A second workspace races an incoming request against initial startup.
    $sessionB=Join-Path $root 'workspace-b'
    $other=Start-TestProcess @('--hidden','--session-dir',(Quote $sessionB),(Quote $names[3]))
    $deadline=[DateTime]::UtcNow.AddSeconds(5)
    while(-not(Test-Path (Join-Path $sessionB 'session.lock')) -and [DateTime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 10}
    Send-Files ($sessionB.ToUpperInvariant()+'\.') @($names[4])
    Wait-Count $sessionB 3
    Wait-Count $sessionA 6
    $checks.Add('Startup handoff resolves the same workspace through case/relative components and stays isolated from other workspaces')
    Stop-Process -Id $primary.Id -Force;$primary.WaitForExit()
    Stop-Process -Id $other.Id -Force;$other.WaitForExit()
    Verify-Session $sessionA 'verify-primary' (@('6','baseline')+$names[0..2])
    Verify-Session $sessionB 'verify-isolation' (@('3','no-baseline')+$names[3..4])
    [IO.File]::WriteAllLines((Join-Path $root 'checks.txt'),$checks,[Text.UTF8Encoding]::new($false))
    $checks
    "PASS: Explorer-style multi-process file-open handoff. Evidence: $root"
}finally{
    foreach($p in $children){if(-not $p.HasExited){Stop-Process -Id $p.Id -Force};$p.Dispose()}
}
