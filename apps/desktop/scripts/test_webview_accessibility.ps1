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
Add-Type -AssemblyName System.Drawing

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

    [StructLayout(LayoutKind.Sequential)]
    private struct Rectangle {
        public int Left, Top, Right, Bottom;
    }

    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hwnd, out Rectangle rect);

    [DllImport("user32.dll")]
    private static extern bool SetForegroundWindow(IntPtr hwnd);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    public static bool ForegroundIsVerifiedHost(IntPtr hwnd) {
        if (hwnd == IntPtr.Zero) return false;
        if (GetForegroundWindow() != hwnd) SetForegroundWindow(hwnd);
        return GetForegroundWindow() == hwnd;
    }

    public static int[][] SamplePoints(IntPtr hwnd) {
        Rectangle rect;
        if (hwnd == IntPtr.Zero || !GetWindowRect(hwnd, out rect) ||
            rect.Right - rect.Left < 10 || rect.Bottom - rect.Top < 10)
            return new int[0][];
        var samples = new List<int[]>();
        foreach (double x in new[] { 0.18, 0.50, 0.82 })
            foreach (double y in new[] { 0.18, 0.50, 0.82 })
                samples.Add(new[] { rect.Left + (int)((rect.Right - rect.Left) * x),
                                    rect.Top + (int)((rect.Bottom - rect.Top) * y) });
        return samples.ToArray();
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct ScreenPoint { public int X, Y; }

    [DllImport("user32.dll")]
    private static extern IntPtr WindowFromPoint(ScreenPoint point);

    [DllImport("user32.dll")]
    private static extern bool IsChild(IntPtr parent, IntPtr child);

    public static bool HandleBelongsToHost(IntPtr parent, IntPtr target) {
        return parent != IntPtr.Zero && target != IntPtr.Zero &&
            (target == parent || IsChild(parent, target));
    }

    public static bool PointBelongsToHost(IntPtr parent, int x, int y) {
        var target = WindowFromPoint(new ScreenPoint { X = x, Y = y });
        return HandleBelongsToHost(parent, target);
    }

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

function Save-BoundedFailureImage {
    # CI launches with a fresh, empty profile and a disposable testnet only.
    # Capture the verified application's foreground rectangle, never the desktop.
    if (-not $env:RUNNER_TEMP) { return }
    $window = Get-AppWindow
    if ($null -eq $window) { return }
    $hwnd = [IntPtr]([long]$window.Current.NativeWindowHandle)
    if (-not [AretovusWebViewChildren]::ForegroundIsVerifiedHost($hwnd)) { return }
    $rect = $window.Current.BoundingRectangle
    $screen = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $left = [Math]::Max([int]$rect.Left, $screen.Left)
    $top = [Math]::Max([int]$rect.Top, $screen.Top)
    $right = [Math]::Min([int]$rect.Right, $screen.Right)
    $bottom = [Math]::Min([int]$rect.Bottom, $screen.Bottom)
    $width = $right - $left
    $height = $bottom - $top
    if ($width -lt 100 -or $height -lt 100 -or
        $width -gt 2000 -or $height -gt 1500) { return }
    $bitmap = [System.Drawing.Bitmap]::new($width, $height)
    try {
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.CopyFromScreen($left, $top, 0, 0, $bitmap.Size)
        } finally { $graphics.Dispose() }
        $destination = Join-Path $env:RUNNER_TEMP 'aretovus-webview-failure.png'
        $bitmap.Save($destination, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally { $bitmap.Dispose() }
}

function Wait-ReactTree {
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    $windowSeen = $false
    $maxElements = 0
    $maxRenderers = 0
    $hitTestCandidates = 0
    $foregroundSeen = $false
    $sampledPoints = 0
    $hostOwnedPoints = 0
    $maxDocuments = 0
    $maxEdits = 0
    $maxButtons = 0
    $maxTexts = 0
    $browserErrorSeen = $false
    $documentDiagnostics = 'none'
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
            # Tauri/wry can leave its Chromium fragment tree orphaned when
            # walked from the host HWND. Windows UIA hit-testing can address
            # the actual renderer provider through its screen point, without
            # changing browser flags or exposing a debugging TCP endpoint.
            # Require the verified Tauri HWND itself to be foreground first,
            # so the hit-test cannot land on some unrelated desktop window.
            if ([AretovusWebViewChildren]::ForegroundIsVerifiedHost($nativeHandle)) {
                $foregroundSeen = $true
                $walker = [System.Windows.Automation.TreeWalker]::RawViewWalker
                foreach ($childHwnd in $rendererHandles) {
                    foreach ($sample in [AretovusWebViewChildren]::SamplePoints($childHwnd)) {
                        $sampledPoints++
                        # Hit-testing must never admit an unrelated window,
                        # even when the verified Tauri host has foreground focus.
                        if (-not [AretovusWebViewChildren]::PointBelongsToHost(
                            $nativeHandle, $sample[0], $sample[1]
                        )) { continue }
                        $hostOwnedPoints++
                        try {
                            $point = [System.Windows.Point]::new([double]$sample[0], [double]$sample[1])
                            $current = [System.Windows.Automation.AutomationElement]::FromPoint($point)
                            for ($depth = 0; $depth -lt 12 -and $null -ne $current; $depth++) {
                                # UIA's raw walker can reach the desktop root;
                                # only accept ancestors native to this Tauri HWND.
                                $candidateHandle = [IntPtr]([long]$current.Current.NativeWindowHandle)
                                if ($candidateHandle -ne [IntPtr]::Zero -and
                                    -not [AretovusWebViewChildren]::HandleBelongsToHost(
                                        $nativeHandle, $candidateHandle
                                    )) { break }
                                $control = $current.Current.ControlType.ProgrammaticName
                                if ($control -eq "ControlType.Document" -or
                                    $control -eq "ControlType.Pane" -or
                                    $control -eq "ControlType.Group" -or
                                    $control -eq "ControlType.Custom") {
                                    $candidates += $current
                                    $hitTestCandidates++
                                }
                                if ($candidateHandle -eq $nativeHandle) { break }
                                $current = $walker.GetParent($current)
                            }
                        } catch [System.Windows.Automation.ElementNotAvailableException] {
                            # Renderer replaced during navigation.
                        } catch [System.Runtime.InteropServices.COMException] {
                            # No usable UIA provider at this host-owned point.
                        }
                    }
                }
            }
            foreach ($root in $candidates) {
                $tree = Get-Elements $root
                $maxElements = [Math]::Max($maxElements, $tree.Count)
                $maxDocuments = [Math]::Max($maxDocuments,
                    @($tree | Where-Object { $_.Type -eq "ControlType.Document" }).Count)
                $maxEdits = [Math]::Max($maxEdits,
                    @($tree | Where-Object { $_.Type -eq "ControlType.Edit" }).Count)
                $maxButtons = [Math]::Max($maxButtons,
                    @($tree | Where-Object { $_.Type -eq "ControlType.Button" }).Count)
                $maxTexts = [Math]::Max($maxTexts,
                    @($tree | Where-Object { $_.Type -eq "ControlType.Text" }).Count)
                # Record only structural properties of the first document. UIA
                # names and values can contain local paths, so never print them.
                if ($documentDiagnostics -eq 'none') {
                    foreach ($entry in $tree) {
                        if ($entry.Type -ne "ControlType.Document") { continue }
                        try {
                            $current = $entry.Element.Current
                            $bounds = $current.BoundingRectangle
                            $documentDiagnostics = "nameLength=$($entry.Name.Length) " +
                                "framework=$($current.FrameworkId) " +
                                "class=$($current.ClassName) " +
                                "offscreen=$($current.IsOffscreen) " +
                                "boundsNonempty=$(-not $bounds.IsEmpty) " +
                                "samePid=$($current.ProcessId -eq $TargetPid)"
                            break
                        } catch [System.Windows.Automation.ElementNotAvailableException] {
                            # Renderer replaced while collecting a diagnostic.
                        }
                    }
                }
                foreach ($hint in @("ERR_FILE_NOT_FOUND", "chrome-error", "can't be reached", "Cannot reach this page")) {
                    if (Match-Name $tree $hint) { $browserErrorSeen = $true }
                }
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
    try { Save-BoundedFailureImage } catch {
        # Screenshot collection must never turn an E2E failure into a success.
    }
    throw "React UIA timeout: windowSeen=$windowSeen maxHostOrRendererElements=$maxElements rendererChildren=$maxRenderers foregroundConfirmed=$foregroundSeen hitTestRoots=$hitTestCandidates sampledPoints=$sampledPoints hostOwnedPoints=$hostOwnedPoints maxDocuments=$maxDocuments maxEdits=$maxEdits maxButtons=$maxButtons maxTexts=$maxTexts browserErrorSeen=$browserErrorSeen documentDiagnostics=$documentDiagnostics missingLandmarks=$($missing -join ',')"
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

function Wait-EnabledButton($window, [int]$seconds, [string]$expectedPath) {
    # A button that was enabled for an OLD path is not evidence that React
    # accepted the newly typed path. Assert the accessible edit value too.
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $tree = Get-Elements $window
        $button = $null
        $pathMatches = $false
        foreach ($entry in $tree) {
            if ($entry.Type -eq "ControlType.Button" -and
                $entry.Name.StartsWith("Connecter le")) {
                $button = $entry
            }
            if ($entry.Type -eq "ControlType.Edit") {
                try {
                    $value = $null
                    if ($entry.Element.TryGetCurrentPattern(
                        [System.Windows.Automation.ValuePattern]::Pattern,
                        [ref]$value
                    )) {
                        $actual = ([System.Windows.Automation.ValuePattern]$value).Current.Value
                        if ($actual -ceq $expectedPath) {
                            $pathMatches = $true
                        }
                    }
                } catch [System.Windows.Automation.ElementNotAvailableException] {
                    # React may replace a control during the input update.
                }
            }
        }
        if ($null -ne $button -and $button.Enabled -and $pathMatches) {
            return $button
        }
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
$button = Wait-EnabledButton $observed.Window 8 $missing
if ($null -eq $button) {
    # Some WebView2 providers implement ValuePattern without dispatching
    # the browser's React onChange event. Use actual focused keyboard input.
    Type-WithKeyboard $input $missing
    $button = Wait-EnabledButton $observed.Window 8 $missing
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
$button = Wait-EnabledButton $observed.Window 12 $TestDataDir
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
