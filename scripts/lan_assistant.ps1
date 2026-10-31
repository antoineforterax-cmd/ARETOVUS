# Assistant Windows pour essais AURORA entre deux ordinateurs du meme LAN prive.
# Ne modifie ni le pare-feu, ni les clefs, ni le consensus.
[CmdletBinding()]
param([switch]$SelfTest)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:Root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$script:Node = Join-Path $script:Root 'bin/aurora-node.exe'
$script:Port = 19444
$script:BeaconPort = 19446
$script:Magic = 'AURORA-LAN-TESTNET-1'

function Test-PrivateIPv4([string]$value) {
    $ip = $null
    if (-not [Net.IPAddress]::TryParse($value, [ref]$ip)) { return $false }
    if ($ip.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork) { return $false }
    $o = $ip.GetAddressBytes()
    return ($o[0] -eq 10 -or ($o[0] -eq 172 -and $o[1] -ge 16 -and $o[1] -le 31) -or ($o[0] -eq 192 -and $o[1] -eq 168))
}
function New-Beacon([string]$hash) {
    if ($hash -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid hash' }
    return "$($script:Magic)|$($hash.ToLowerInvariant())|$($script:Port)"
}
function Read-Beacon([string]$message, [string]$source, [string]$myHash) {
    if ($message.Length -gt 180 -or -not (Test-PrivateIPv4 $source)) { return $null }
    $parts = $message.Split('|')
    if ($parts.Length -ne 3 -or $parts[0] -cne $script:Magic) { return $null }
    if ($parts[1] -cnotmatch '^[a-f0-9]{64}$' -or $parts[1] -cne $myHash.ToLowerInvariant()) { return $null }
    if ($parts[2] -cne "$($script:Port)") { return $null }
    return $source
}
function New-NodeArgs([string]$role, [string]$address = '') {
    if ($role -eq 'A' -and (Test-PrivateIPv4 $address)) {
        return @('testnet','serve','--data-dir',(Join-Path $script:Root 'two-pc/assistant-a'),'--listen',"$($address):19444")
    }
    if ($role -eq 'B' -and (Test-PrivateIPv4 $address)) {
        return @('testnet','serve','--data-dir',(Join-Path $script:Root 'two-pc/assistant-b'),'--listen','127.0.0.1:19445','--peer',"$($address):19444")
    }
    throw 'Invalid role or private IPv4 peer'
}
function Get-PrivateBindIp {
    $active = @(Get-NetConnectionProfile -ErrorAction Stop | Where-Object { $_.IPv4Connectivity -ne 'Disconnected' })
    if ($active.Count -eq 0 -or @($active | Where-Object { $_.NetworkCategory -ne 'Private' }).Count -gt 0) {
        throw 'Un profil Windows Public actif est present, ou aucun reseau Prive actif. Connexion refusee.'
    }
    $ips = @(foreach ($profile in $active) {
        Get-NetIPAddress -InterfaceIndex $profile.InterfaceIndex -AddressFamily IPv4 -ErrorAction Stop |
            Where-Object { Test-PrivateIPv4 $_.IPAddress } | Select-Object -ExpandProperty IPAddress
    } | Select-Object -Unique)
    if ($ips.Count -ne 1) { throw 'Plusieurs reseaux IP prives actifs, ou aucun. Deconnecter VPN/adaptateurs supplementaires avant utilisation.' }
    return [string]$ips[0]
}
function Test-Tcp([string]$ip) {
    if (-not (Test-PrivateIPv4 $ip)) { return $false }
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $async = $client.BeginConnect($ip, $script:Port, $null, $null)
        try {
            if (-not $async.AsyncWaitHandle.WaitOne(1500)) { return $false }
            $client.EndConnect($async)
            return $true
        } finally { $async.AsyncWaitHandle.Close() }
    } catch { return $false }
    finally { $client.Close() }
}
function Test-Package {
    $manifest = Join-Path $script:Root 'SHA256SUMS.txt'
    foreach ($file in @($script:Node,$manifest,(Join-Path $script:Root 'EXPERIMENTAL_TESTNET.txt'),(Join-Path $script:Root 'bin/aurora-wallet.exe'))) {
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Paquet incomplet : $file" }
    }
    $matches = @(Get-Content -LiteralPath $manifest | Where-Object { $_ -match '^[a-f0-9]{64}  bin/aurora-node.exe$' })
    if ($matches.Count -ne 1) { throw 'Somme de controle du noeud absente' }
    $sha = (Get-FileHash -LiteralPath $script:Node -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sha -cne $matches[0].Substring(0,64)) { throw 'Binaire non conforme aux sommes de controle' }
    & (Join-Path $script:Root 'scripts/two_pc_preflight.ps1') -PackageRoot $script:Root -Offline | Out-Null
    return $sha
}
if ($SelfTest) {
    foreach ($ip in @('10.1.2.3','172.16.0.1','172.31.255.254','192.168.1.1')) {
        if (-not (Test-PrivateIPv4 $ip)) { throw "Private IPv4 rejected: $ip" }
    }
    foreach ($ip in @('8.8.8.8','127.0.0.1','172.32.0.1','169.254.1.1','::1','evil.example','192.168.1.2;x')) {
        if (Test-PrivateIPv4 $ip) { throw "Unsafe IPv4 accepted: $ip" }
    }
    $hash = 'a' * 64
    if ((Read-Beacon (New-Beacon $hash) '192.168.1.2' $hash) -cne '192.168.1.2') { throw 'Beacon roundtrip failed' }
    if ($null -ne (Read-Beacon (New-Beacon $hash) '192.168.1.2' ('b' * 64))) { throw 'Hash mismatch accepted' }
    if ($null -ne (Read-Beacon (New-Beacon $hash) '8.8.8.8' $hash)) { throw 'Public peer accepted' }
    if ($null -ne (Read-Beacon 'bad' '192.168.1.2' $hash)) { throw 'Bad beacon accepted' }
    if ((New-NodeArgs 'A' '192.168.1.5')[-1] -cne '192.168.1.5:19444') { throw 'A arguments wrong' }
    if ((New-NodeArgs 'B' '192.168.1.2')[-1] -cne '192.168.1.2:19444') { throw 'B arguments wrong' }
    try { $null = New-NodeArgs 'B' '8.8.8.8'; throw 'Public peer accepted as B' }
    catch { if ($_.Exception.Message -eq 'Public peer accepted as B') { throw } }
    Write-Output 'lan_assistant_selftest=ok'
    exit 0
}
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[System.Windows.Forms.Application]::EnableVisualStyles()
$form = New-Object System.Windows.Forms.Form
$form.Text = 'AURORA - Assistant de connexion LAN (testnet)'
$form.Size = New-Object System.Drawing.Size(670,430)
$form.StartPosition = 'CenterScreen'
$form.FormBorderStyle = 'FixedDialog'
$form.MaximizeBox = $false
$form.Font = New-Object System.Drawing.Font('Segoe UI',10)
function Label([string]$txt,[int]$x,[int]$y,[int]$w,[int]$h=38) {
    $obj = New-Object System.Windows.Forms.Label
    $obj.Text=$txt
    $obj.Location=New-Object System.Drawing.Point($x,$y)
    $obj.Size=New-Object System.Drawing.Size($w,$h)
    $form.Controls.Add($obj)
    return $obj
}
function Button([string]$txt,[int]$x,[int]$y,[int]$w) {
    $obj = New-Object System.Windows.Forms.Button
    $obj.Text=$txt
    $obj.Location=New-Object System.Drawing.Point($x,$y)
    $obj.Size=New-Object System.Drawing.Size($w,42)
    $form.Controls.Add($obj)
    return $obj
}
function Alert([string]$message) {
    [System.Windows.Forms.MessageBox]::Show($message,'AURORA LAN','OK','Warning') | Out-Null
}
$warning = Label 'TESTNET experimental. Reseau Prive uniquement. Jamais de fonds reels.' 18 15 615
$warning.ForeColor = [System.Drawing.Color]::DarkRed
$description = Label 'PC A heberge. PC B le detecte et se synchronise sans recopier son adresse.' 18 55 615
$start = Button 'PC A : demarrer le serveur' 18 105 285
$join = Button 'PC B : connexion automatique' 330 105 300
$ipText = Label 'Si la decouverte est bloquee, saisir ici l IP privee de PC A :' 18 161 615
$ip = New-Object System.Windows.Forms.TextBox
$ip.Location = New-Object System.Drawing.Point(18,193)
$ip.Width=250
$form.Controls.Add($ip)
$manual = Button 'Connecter B avec cette adresse' 286 185 344
$firewall = Label 'Pare-feu : TCP 19444 autorise uniquement pour PC B sur profil Prive, jamais Internet.' 18 250 615 48
$firewall.ForeColor = [System.Drawing.Color]::DarkBlue
$status = Label 'Verification du paquet et de la decouverte...' 18 300 615 44
$stop = Label 'Pour arreter proprement, utilisez Ctrl+C dans la fenetre de chaque noeud.' 18 355 615 34
$script:Hash = $null
$script:Proc = $null
$script:Role = ''
$script:Requested = $false
$script:Discovered = ''
$script:Listener = $null
$script:Sender = $null
$script:LastBroadcast = [DateTime]::MinValue
$script:Warned = $false
$script:BindIp = ''
function Start-Node([string]$role,[string]$address='') {
    if ($null -ne $script:Proc -and -not $script:Proc.HasExited) { throw 'Un noeud fonctionne deja. Arretez-le avec Ctrl+C.' }
    $localIp = Get-PrivateBindIp
    if ($role -eq 'A') { $address = $localIp; $script:BindIp = $localIp }
    $nodeArgs = @(New-NodeArgs $role $address)
    # Paths cannot contain a quote on Windows. Each CLI argument is quoted; no cmd /c interpolation.
    $quoted = @($nodeArgs | ForEach-Object { '"' + $_.Replace('"','') + '"' })
    $script:Proc = Start-Process -FilePath $script:Node -ArgumentList $quoted -WorkingDirectory $script:Root -PassThru
    $script:Role = $role
    $script:Requested = $false
    $start.Enabled=$false
    $join.Enabled=$false
    $manual.Enabled=$false
    $status.Text = "PC $role : noeud lance dans une autre fenetre. Verifier la synchronisation."
}
function Connect-B {
    $hostIp = $ip.Text.Trim()
    if (-not (Test-PrivateIPv4 $hostIp)) { throw 'Adresse IPv4 privee requise.' }
    if (-not (Test-Tcp $hostIp)) { throw 'PC A inaccessible en TCP 19444. Verifier pare-feu, LAN et serveur.' }
    Start-Node 'B' $hostIp
}
$start.Add_Click({ try { Start-Node 'A' } catch { Alert $_.Exception.Message } })
$join.Add_Click({
    try {
        $null = Get-PrivateBindIp
        $script:Requested=$true
        $status.Text='Recherche de PC A sur le LAN...'
        if ($script:Discovered) { $ip.Text=$script:Discovered; Connect-B }
    } catch { $script:Requested=$false; Alert $_.Exception.Message }
})
$manual.Add_Click({ try { Connect-B } catch { Alert $_.Exception.Message } })
try {
    $script:Hash = Test-Package
    $status.Text='Paquet verifie. Selectionner PC A ou PC B.'
    $bind = Get-PrivateBindIp
    $script:Sender=New-Object System.Net.Sockets.UdpClient
    $script:Sender.Client.Bind([Net.IPEndPoint]::new([Net.IPAddress]::Parse($bind),0))
    $script:Sender.EnableBroadcast=$true
    $script:Listener=New-Object System.Net.Sockets.UdpClient
    $script:Listener.Client.ExclusiveAddressUse=$false
    $script:Listener.Client.SetSocketOption([Net.Sockets.SocketOptionLevel]::Socket,[Net.Sockets.SocketOptionName]::ReuseAddress,$true)
    $endpoint=[Net.IPEndPoint]::new([Net.IPAddress]::Any,$script:BeaconPort)
    $script:Listener.Client.Bind($endpoint)
} catch {
    $start.Enabled=$false; $join.Enabled=$false; $manual.Enabled=$false
    $status.Text='Preflight impossible. Corriger le paquet ou le reseau et relancer.'
    Alert $_.Exception.Message
}
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval=850
$timer.Add_Tick({
    if ($null -eq $script:Listener -or $null -eq $script:Hash) { return }
    if ($null -ne $script:Proc -and $script:Proc.HasExited) {
        $status.Text="Processus termine (code $($script:Proc.ExitCode)). Cela ne prouve pas la convergence."
        $script:Proc=$null; $script:Role=''
        $start.Enabled=$true; $join.Enabled=$true; $manual.Enabled=$true
    }
    try {
        if ($script:Role -eq 'A' -and ([DateTime]::UtcNow-$script:LastBroadcast).TotalSeconds -gt 2) {
            $beacon=[Text.Encoding]::ASCII.GetBytes((New-Beacon $script:Hash))
            $script:Sender.Send($beacon,$beacon.Length,'255.255.255.255',$script:BeaconPort) | Out-Null
            $script:LastBroadcast=[DateTime]::UtcNow
        }
        for($i=0;$i -lt 12 -and $script:Listener.Available -gt 0;$i++){
            $remote=[Net.IPEndPoint]::new([Net.IPAddress]::Any,0)
            $packet=$script:Listener.Receive([ref]$remote)
            if($packet.Length -gt 180){continue}
            $found=Read-Beacon ([Text.Encoding]::ASCII.GetString($packet)) $remote.Address.ToString() $script:Hash
            if(-not $found){continue}
            $script:Discovered=$found
            $ip.Text=$found
            if($script:Requested){
                $script:Requested=$false
                try {Connect-B} catch {$status.Text='PC A detecte, mais la connexion TCP a echoue.';Alert $_.Exception.Message}
            } elseif(-not $script:Role){
                $status.Text="PC A decouvert : $found. Cliquer sur connexion automatique."
            }
        }
    } catch {
        if(-not $script:Warned){
            $script:Warned=$true
            $status.Text='Decouverte UDP indisponible. Utiliser l IP privee de PC A.'
        }
    }
})
$timer.Start()
$form.Add_FormClosing({
    if($null -ne $script:Proc -and -not $script:Proc.HasExited){
        $answer=[System.Windows.Forms.MessageBox]::Show('Le noeud est encore actif. Arretez-le avec Ctrl+C dans son terminal. Fermer cette interface sans arreter le noeud ?', 'Noeud actif', 'YesNo', 'Warning')
        if($answer -ne [System.Windows.Forms.DialogResult]::Yes){$_.Cancel=$true}
    }
})
$form.Add_FormClosed({
    $timer.Stop()
    if($null -ne $script:Listener){$script:Listener.Close()}
    if($null -ne $script:Sender){$script:Sender.Close()}
})
[System.Windows.Forms.Application]::Run($form)
