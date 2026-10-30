# Verify an unpacked AURORA CI package before a private-LAN trial.
# This script never changes network settings or opens a firewall port.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $PackageRoot,

    [string] $ExpectedRevision,

    [string] $PeerAddress,

    [ValidateRange(1, 65535)]
    [int] $PeerPort = 19444,

    [switch] $Offline
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath($PackageRoot)
$node = Join-Path $root 'bin/aurora-node.exe'
$wallet = Join-Path $root 'bin/aurora-wallet.exe'
$manifest = Join-Path $root 'EXPERIMENTAL_TESTNET.txt'
$checksums = Join-Path $root 'SHA256SUMS.txt'

foreach ($file in @($node, $wallet, $manifest, $checksums)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
        throw "Missing AURORA package file: $file"
    }
}

$stamp = Get-Content -LiteralPath $manifest -Raw
$revisionMatch = [regex]::Match($stamp, '(?m)^Source commit: ([0-9a-f]{12})\r?$')
if (-not $revisionMatch.Success) {
    throw 'Missing or malformed source revision in EXPERIMENTAL_TESTNET.txt'
}
$revision = $revisionMatch.Groups[1].Value
if ($ExpectedRevision -and $ExpectedRevision -ne $revision) {
    throw "Wrong build revision: got $revision, expected $ExpectedRevision"
}

$count = 0
$rootPrefix = $root.TrimEnd([System.IO.Path]::DirectorySeparatorChar) +
    [System.IO.Path]::DirectorySeparatorChar
foreach ($line in Get-Content -LiteralPath $checksums) {
    if (-not $line) { continue }
    if ($line -notmatch '^([0-9a-fA-F]{64})  (.+)$') {
        throw "Malformed checksum line: $line"
    }
    $expected = $Matches[1].ToLowerInvariant()
    $entry = $Matches[2]
    $file = [System.IO.Path]::GetFullPath(
        (Join-Path $root ($entry.Replace('/', [System.IO.Path]::DirectorySeparatorChar)))
    )
    if (-not $file.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Checksum entry escapes package root: $entry"
    }
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
        throw "Missing checksummed file: $entry"
    }
    $actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        throw "Checksum mismatch for $entry"
    }
    $count++
}
if ($count -lt 2) {
    throw 'Package has no usable checksums'
}

# A run without credentials or a wallet is enough to check executable startup.
& $node --help | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'aurora-node.exe --help failed' }
& $wallet --help | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'aurora-wallet.exe --help failed' }

Write-Host "AURORA package OK: revision=$revision checked_files=$count"
Write-Host "node_sha256=$((Get-FileHash -LiteralPath $node -Algorithm SHA256).Hash)"
Write-Host "wallet_sha256=$((Get-FileHash -LiteralPath $wallet -Algorithm SHA256).Hash)"

if ($Offline) {
    Write-Host 'Offline check completed; LAN connectivity has not been tested.'
    exit 0
}
if (-not $PeerAddress) {
    throw 'Provide -PeerAddress for a LAN check, or -Offline for package verification only.'
}

$ip = $null
if (-not [System.Net.IPAddress]::TryParse($PeerAddress, [ref] $ip) -
    or $ip.AddressFamily -ne [System.Net.Sockets.AddressFamily]::InterNetwork) {
    throw 'PeerAddress must be a numeric IPv4 address.'
}
$octets = $ip.GetAddressBytes()
$isPrivate = $octets[0] -eq 10 -or
    ($octets[0] -eq 172 -and $octets[1] -ge 16 -and $octets[1] -le 31) -or
    ($octets[0] -eq 192 -and $octets[1] -eq 168)
if (-not $isPrivate) {
    throw 'PeerAddress must be a private LAN IPv4 address; public peers are not supported.'
}
if (Get-Command Get-NetConnectionProfile -ErrorAction SilentlyContinue) {
    $profiles = @(Get-NetConnectionProfile)
    if ($profiles | Where-Object { $_.NetworkCategory -eq 'Public' }) {
        Write-Warning 'At least one Windows network profile is Public; use a trusted Private LAN.'
    }
}
if (-not (Test-NetConnection -ComputerName $PeerAddress -Port $PeerPort -InformationLevel Quiet -WarningAction SilentlyContinue)) {
    throw "TCP connection failed to $PeerAddress`:$PeerPort. Start the peer and check PRIVATE firewall rules."
}
Write-Host "Private LAN connection OK: $PeerAddress`:$PeerPort"
