#requires -Version 5.1
<#
Windows-only read-only smoke of the ACTUAL embedded React content in WebView2.
Requires UI Automation provider and an interactive CI desktop; failure is blocking.
No browser debugging port, WebDriver service, secrets or network listener.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateRange(1, 2147483647)]
    [int] $TargetPid
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms

function Get-AppWindow {
    $cond = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        [int] $TargetPid
    )
    return [System.Windows.Automation.AutomationElement]::RootElement.FindFirst(
        [System.Windows.Automation.TreeScope]::Children, $cond
    )
}

function Get-Elements($root) {
    $collection = $root.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.Condition]::TrueCondition
    )
    $result = New-Object System.Collections.Generic.List[object]
    for ($i = 0; $i -lt $collection.Count; $i++) {
        try {
            $element = $collection.Item($i)
            $name = [string]$element.Current.Name
            $type = $element.Current.ControlType.ProgrammaticName
            $enabled = [bool]$element.Current.IsEnabled
            $result.Add([pscustomobject]@{
                Element = $element
                Name = $name
                Type = $type
                Enabled = $enabled
            })
        } catch [System.Windows.Automation.ElementNotAvailableException] {
            # Accessibility tree can change during React rendering.
        }
    }
    return ,$result
}

function Match-Name($items, [string]$text) {
    foreach ($entry in $items) {
        if ($entry.Name.Contains($text)) { return $true }
    }
    return $false
}

function Wait-ReactTree {
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    while ([DateTime]::UtcNow -lt $deadline) {
        $process = [System.Diagnostics.Process]::GetProcessById($TargetPid)
        if ($process.HasExited) { throw "GUI exited before React accessibility tree loaded" }
        $window = Get-AppWindow
        if ($null -ne $window) {
            $tree = Get-Elements $window
            if (
                (Match-Name $tree "Explorateur de blocs") -and
                (Match-Name $tree "Hauteur valid") -and
                (Match-Name $tree "TESTNET") -and
                (Match-Name $tree "Aucun n") -and
                (Match-Name $tree "Connecter le")
            ) {
                return @{ Window = $window; Tree = $tree }
            }
        }
        Start-Sleep -Milliseconds 350
    }
    throw "Embedded React content was NOT exposed by WebView2 UI Automation within 45s"
}

function Find-ByNameAndType($tree, [string]$prefix, [string]$type) {
    foreach ($entry in $tree) {
        if ($entry.Type -eq $type -and $entry.Name.StartsWith($prefix)) {
            return $entry
        }
    }
    throw "Missing accessible element of type $type with name prefix $prefix"
}

function Set-InputValue($inputElement, [string]$value) {
    $pattern = $null
    if ($inputElement.TryGetCurrentPattern(
        [System.Windows.Automation.ValuePattern]::Pattern, [ref]$pattern
    )) {
        $valuePattern = [System.Windows.Automation.ValuePattern]$pattern
        if (-not $valuePattern.Current.IsReadOnly) {
            $valuePattern.SetValue($value)
            return
        }
    }
    # Real keyboard path for WebView2 accessibility providers that expose an
    # Edit control without a writable ValuePattern. No mock DOM injection.
    $inputElement.SetFocus()
    [System.Windows.Forms.SendKeys]::SendWait($value)
}

function Activate-Button($buttonElement) {
    $pattern = $null
    if ($buttonElement.TryGetCurrentPattern(
        [System.Windows.Automation.InvokePattern]::Pattern, [ref]$pattern
    )) {
        ([System.Windows.Automation.InvokePattern]$pattern).Invoke()
        return
    }
    $buttonElement.SetFocus()
    [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
}

$observed = Wait-ReactTree
$tree = $observed.Tree
$input = $null
foreach ($entry in $tree) {
    if ($entry.Type -eq "ControlType.Edit") { $input = $entry.Element; break }
}
if ($null -eq $input) { throw "React form input is not accessible as a WebView2 Edit control" }
$button = Find-ByNameAndType $tree "Connecter le" "ControlType.Button"
if ($button.Enabled) { throw "Connect button should be disabled for an empty path" }

# Intentionally nonexistent (random) testnet folder; no keys, files or services.
$missing = "X:\ARETOVUS_MISSING_$([Guid]::NewGuid().ToString('N'))"
Set-InputValue $input $missing
$deadline = [DateTime]::UtcNow.AddSeconds(12)
$buttonReady = $false
while ([DateTime]::UtcNow -lt $deadline) {
    $tree = Get-Elements $observed.Window
    $button = Find-ByNameAndType $tree "Connecter le" "ControlType.Button"
    if ($button.Enabled) { $buttonReady = $true; break }
    Start-Sleep -Milliseconds 250
}
if (-not $buttonReady) {
    throw "Typing a nonexistent path did not enable the React-controlled button"
}
Activate-Button $button.Element
$deadline = [DateTime]::UtcNow.AddSeconds(30)
while ([DateTime]::UtcNow -lt $deadline) {
    $tree = Get-Elements $observed.Window
    if ((Match-Name $tree "Lecture impossible") -and (Match-Name $tree "N") ) {
        # Explicit error from the native Tauri IPC boundary, not invented data.
        Write-Output "PASS: WebView2 exposes React dashboard and native IPC error for nonexistent testnet path"
        Write-Output "NOT CHECKED: valid snapshot status, explorer pagination, screen pixels, wallet, mainnet"
        exit 0
    }
    Start-Sleep -Milliseconds 350
}
throw "React did not expose native invalid-path error within 30s after UI interaction"
