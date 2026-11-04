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
    [int] $TargetPid,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string] $TestDataDir
)
$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $TestDataDir -PathType Container)) {
    throw "Test data directory is missing or not a directory"
}
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms

# WebView2 renders inside a separate Chromium process. On some Tauri/wry
# versions, the top-level host HWND's UIA tree does not expose its WebView2
# content. Find child render HWNDs belonging to the VERIFIED Tauri host
# window; then ask Windows UIA for those real native elements directly.
# This does not start a debug listener or inject JavaScript.
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class AretovusWebViewChildren {
    private delegate bool EnumChildrenProc(IntPtr hwnd, IntPtr lparam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumChildWindows(
        IntPtr parent, EnumChildrenProc callback, IntPtr lparam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetClassName(
        IntPtr hwnd, StringBuilder name, int capacity);

    public static IntPtr[] Find(IntPtr parent) {
        var renderers = new List<IntPtr>();
        var webviews = new List<IntPtr>();
        if (parent == IntPtr.Zero) return renderers.ToArray();
        EnumChildWindows(parent, delegate(IntPtr hwnd, IntPtr unused) {
            var name = new StringBuilder(128);
            if (GetClassName(hwnd, name, name.Capacity) > 0) {
                if (name.ToString() == "Chrome_RenderWidgetHostHWND")
                    renderers.Add(hwnd);
                else if (name.ToString() == "Chrome_WidgetWin_0")
                    webviews.Add(hwnd);
            }
            return true;
        }, IntPtr.Zero);
        renderers.AddRange(webviews);
        return renderers.ToArray();
    }
}
'@

function Get-AppWindow {
    $cond = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty,
        [int] $TargetPid
    )
    # A native host can own more than one top-level HWND. FindFirst can pick
    # a hidden auxiliary window instead of the visible Tauri window already
    # verified by the independent Win32 test.
    $windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll(
        [System.Windows.Automation.TreeScope]::Children, $cond
    )
    for ($i = 0; $i -lt $windows.Count; $i++) {
        try {
            $window = $windows.Item($i)
            if ($window.Current.Name -eq ("ARETOVUS " + [char]0x2014 + " Node Desktop") -and
                -not $window.Current.IsOffscreen) {
                return $window
            }
        } catch [System.Windows.Automation.ElementNotAvailableException] {
            # Window disposed while the WebView2 process starts.
        }
    }
    return $null
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
    $windowSeen = $false
    $maxElements = 0
    $maxRenderers = 0
    $seen = @{}
    $landmarks = @("Explorateur de blocs", "Hauteur valid", "TESTNET", "Aucun n", "Connecter le")
    foreach ($landmark in $landmarks) { $seen[$landmark] = $false }

    while ([DateTime]::UtcNow -lt $deadline) {
        $process = [System.Diagnostics.Process]::GetProcessById($TargetPid)
        if ($process.HasExited) { throw "GUI exited before React accessibility tree loaded" }
        $window = Get-AppWindow
        if ($null -ne $window) {
            $windowSeen = $true
            $candidates = @($window)
            $nativeHandle = [IntPtr]([long]$window.Current.NativeWindowHandle)
            $rendererHandles = [AretovusWebViewChildren]::Find($nativeHandle)
            $maxRenderers = [Math]::Max($maxRenderers, $rendererHandles.Count)
            foreach ($childHwnd in $rendererHandles) {
                try {
                    $candidate = [System.Windows.Automation.AutomationElement]::FromHandle($childHwnd)
                    if ($null -ne $candidate) { $candidates += $candidate }
                } catch [System.Windows.Automation.ElementNotAvailableException] {
                    # Chromium child was destroyed while the view started.
                } catch [System.Runtime.InteropServices.COMException] {
                    # A transient child HWND may lack a UIA provider.
                }
            }
            foreach ($root in $candidates) {
                $tree = Get-Elements $root
                $maxElements = [Math]::Max($maxElements, $tree.Count)
                foreach ($landmark in $landmarks) {
                    if (Match-Name $tree $landmark) { $seen[$landmark] = $true }
                }
                if (
                    (Match-Name $tree "Explorateur de blocs") -and
                    (Match-Name $tree "Hauteur valid") -and
                    (Match-Name $tree "TESTNET") -and
                    (Match-Name $tree "Aucun n") -and
                    (Match-Name $tree "Connecter le")
                ) {
                    return @{ Window = $root; Tree = $tree }
                }
            }
        }
        Start-Sleep -Milliseconds 350
    }
    # Do not dump UIA element text: text may contain sensitive local paths.
    # Only disclose presence/absence of five fixed labels and bounded counts.
    $missing = @($landmarks | Where-Object { -not $seen[$_] })
    throw "React UIA timeout: windowSeen=$windowSeen maxHostOrRendererElements=$maxElements rendererChildren=$maxRenderers missingLandmarks=$($missing -join ',')"
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

function Type-WithKeyboard($inputElement, [string]$value) {
    $inputElement.SetFocus()
    [System.Windows.Forms.SendKeys]::SendWait("^a")
    [System.Windows.Forms.SendKeys]::SendWait($value)
}

function Wait-EnabledButton($window, [int]$seconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $tree = Get-Elements $window
        $button = Find-ByNameAndType $tree "Connecter le" "ControlType.Button"
        if ($button.Enabled) { return $button }
        Start-Sleep -Milliseconds 250
    }
    return $null
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
$button = Wait-EnabledButton $observed.Window 8
if ($null -eq $button) {
    # Some WebView2 providers implement ValuePattern without dispatching
    # the browser's React onChange event. Use actual focused keyboard input.
    Type-WithKeyboard $input $missing
    $button = Wait-EnabledButton $observed.Window 8
}
if ($null -eq $button) {
    throw "Typing a nonexistent path did not enable the React-controlled button"
}
Activate-Button $button.Element
$deadline = [DateTime]::UtcNow.AddSeconds(30)
while ([DateTime]::UtcNow -lt $deadline) {
    $tree = Get-Elements $observed.Window
    if ((Match-Name $tree "Lecture impossible") -and (Match-Name $tree "indisponible")) {
        # Native Tauri host correctly rejected the absent folder.
        break
    }
    Start-Sleep -Milliseconds 350
}
if (-not ((Match-Name $tree "Lecture impossible") -and (Match-Name $tree "indisponible"))) {
    throw "Native invalid-path rejection was never exposed in React"
}

# Now switch to a newly mined, disposable local testnet snapshot, created by
# the Python parent with the REAL node CLI from the same verified ZIP.
# React has re-rendered after the invalid-path error; reacquire the Edit element
# instead of holding an accessibility handle that might have become stale.
$tree = Get-Elements $observed.Window
$input = $null
foreach ($entry in $tree) {
    if ($entry.Type -eq "ControlType.Edit") { $input = $entry.Element; break }
}
if ($null -eq $input) { throw "React path control disappeared after native error" }
Type-WithKeyboard $input $TestDataDir
$button = Wait-EnabledButton $observed.Window 12
if ($null -eq $button) { throw "Could not select disposable native testnet directory" }
Activate-Button $button.Element

$deadline = [DateTime]::UtcNow.AddSeconds(45)
$ready = $false
while ([DateTime]::UtcNow -lt $deadline) {
    $tree = Get-Elements $observed.Window
    if ((Match-Name $tree "Snapshot local valid") -and (Match-Name $tree "TESTNET")) {
        $ready = $true
        break
    }
    Start-Sleep -Milliseconds 350
}
if (-not $ready) { throw "Tauri/React never displayed a validated real local testnet snapshot" }

# Explicit on-demand native explorer call, not a fake page or mocked network.
$load = Find-ByNameAndType $tree "Lire les blocs" "ControlType.Button"
if (-not $load.Enabled) { throw "Native explorer button remains disabled after valid snapshot" }
Activate-Button $load.Element
$deadline = [DateTime]::UtcNow.AddSeconds(45)
while ([DateTime]::UtcNow -lt $deadline) {
    $tree = Get-Elements $observed.Window
    if ((Match-Name $tree "Hash du bloc") -and (Match-Name $tree "Timestamp (s)")) {
        Write-Output "PASS: WebView2 React displayed real Rust snapshot and canonical explorer from extracted ZIP"
        Write-Output "PASS: Invalid-path native rejection, controlled path edit, and read-only explorer click"
        Write-Output "NOT CHECKED: screen pixels, all accessibility modes, real funds, wallet GUI or P2P live"
        exit 0
    }
    Start-Sleep -Milliseconds 350
}
throw "Native explorer did not render canonical block table after real UI interaction"
