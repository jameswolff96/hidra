param([switch]$Release)
$cfg = if ($Release) { "release" } else { "debug" }
Write-Host "Building ($cfg)..."
cargo build --workspace --$cfg
