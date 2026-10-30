param([switch]$Test,[switch]$Debug,[switch]$Clippy,[string]$ToolingRoot,[string]$Output='dist\ByteForge-Rust.exe')
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
if(-not $ToolingRoot){$ToolingRoot=Join-Path $projectRoot '..\CSVcoder-Rust\.tooling'}
$localCargo=Join-Path $ToolingRoot 'cargo\bin\cargo.exe'
if(Test-Path -LiteralPath $localCargo){
    $env:CARGO_HOME=[IO.Path]::GetFullPath((Join-Path $ToolingRoot 'cargo'))
    $env:RUSTUP_HOME=[IO.Path]::GetFullPath((Join-Path $ToolingRoot 'rustup'))
    $env:PATH=(Split-Path $localCargo)+';'+$env:PATH
    $sysroot=& (Join-Path (Split-Path $localCargo) 'rustc.exe') --print sysroot
    $env:PATH=(Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained')+';'+$env:PATH
    $cargo=$localCargo
}else{$cargo=(Get-Command cargo -ErrorAction Stop).Source}
Push-Location $projectRoot
try{
    if(-not(Test-Path Cargo.lock)){& $cargo generate-lockfile --offline; if($LASTEXITCODE){throw 'Lockfile generation failed'}}
    & $cargo fmt -- --check; if($LASTEXITCODE){throw 'Formatting check failed'}
    if($Test){& $cargo test --locked --offline; if($LASTEXITCODE){throw 'Tests failed'}}
    if($Clippy){& $cargo clippy --all-targets --locked --offline -- -D warnings; if($LASTEXITCODE){throw 'Clippy failed'}}
    $buildArgs=@('build','--locked','--offline');$profile='debug'
    if(-not $Debug){$buildArgs+='--release';$profile='release'}
    & $cargo @buildArgs; if($LASTEXITCODE){throw 'Build failed'}
    New-Item -ItemType Directory -Path dist -Force|Out-Null
    $outputPath=$ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Output)
    New-Item -ItemType Directory -Path (Split-Path -Parent $outputPath) -Force|Out-Null
    Copy-Item "target\$profile\ByteForge-Rust.exe" -Destination $outputPath
    Get-Item $outputPath|Select-Object FullName,Length
    Get-FileHash $outputPath -Algorithm SHA256|Format-List
}finally{Pop-Location}
