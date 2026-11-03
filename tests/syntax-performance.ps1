param([string]$Executable="$PSScriptRoot\..\dist\ByteForge-Rust.exe",[string]$OutputDirectory,[string]$InputFile)
$ErrorActionPreference='Stop'
$exe=(Resolve-Path $Executable).Path
if(-not $OutputDirectory){$OutputDirectory=Join-Path (Split-Path $PSScriptRoot -Parent) ('output\syntax-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))}
$root=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $root){throw 'Use a new syntax QA directory.'}
New-Item -ItemType Directory -Path $root|Out-Null
if($InputFile){$inputs=@((Resolve-Path -LiteralPath $InputFile).Path)}else{
    $rows=for($i=0;$i -lt 320;$i++){'{"type":"FINDING","id":'+$i+',"description":"'+('synthetic sample '*100)+'","ready":true}'}
    $ndjson=Join-Path $root 'long-rows.ndjson';[IO.File]::WriteAllText($ndjson,($rows -join "`n"),[Text.UTF8Encoding]::new($false))
    $members=for($i=0;$i -lt 25000;$i++){'"key'+$i+'":"value'+$i+'"'}
    $minified=Join-Path $root 'minified.json';[IO.File]::WriteAllText($minified,('{'+($members -join ',')+'}'),[Text.UTF8Encoding]::new($false))
    $inputs=@($ndjson,$minified)
}
foreach($inputPath in $inputs){
    $out=Join-Path $root ([IO.Path]::GetFileNameWithoutExtension($inputPath)+'-result')
    $p=Start-Process -FilePath $exe -ArgumentList @('--hidden','--profile-syntax',('"'+$out+'"'),'--profile-input',('"'+$inputPath+'"')) -WindowStyle Hidden -PassThru
    try{
        if(-not $p.WaitForExit(20000)){throw 'Syntax performance regression: test process exceeded 20 seconds.'}
        $result=[IO.File]::ReadAllText((Join-Path $out 'result.txt'))
        if(-not $result.StartsWith('PASS:')){throw $result}
        Get-Content (Join-Path $out 'timings.txt')
        $result.Trim()
    }finally{if(-not $p.HasExited){Stop-Process -Id $p.Id -Force};$p.Dispose()}
}
"Evidence: $root"
