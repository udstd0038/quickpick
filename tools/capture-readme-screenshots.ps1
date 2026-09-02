param(
  [string]$QuickPickExe = "",
  [string]$OutputRoot = "",
  [string[]]$Themes = @("light"),
  [switch]$IncludeScreenshot
)

$ErrorActionPreference = "Stop"

if (-not $QuickPickExe) {
  $QuickPickExe = Join-Path $PSScriptRoot "..\apps\desktop\src-tauri\target-monorepo-check\release\quickpick.exe"
}

if (-not $OutputRoot) {
  $OutputRoot = Join-Path $PSScriptRoot ".."
}

$QuickPickExe = [IO.Path]::GetFullPath($QuickPickExe)
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)

if (-not (Test-Path -LiteralPath $QuickPickExe)) {
  throw "QuickPick release binary was not found at $QuickPickExe"
}

Add-Type -AssemblyName System.Drawing

Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;

public struct RECT {
  public int Left;
  public int Top;
  public int Right;
  public int Bottom;
}

public static class QuickPickCapture {
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
  [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
  [DllImport("kernel32.dll")] public static extern IntPtr GetConsoleWindow();
}
'@

[QuickPickCapture]::SetProcessDPIAware() | Out-Null

function Get-ProcessWindowRows([uint32]$ProcessId) {
  $script:ProcessId = $ProcessId
  $rows = [System.Collections.Generic.List[object]]::new()
  $script:rows = $rows
  [QuickPickCapture]::EnumWindows({
    param($h, $l)
    [uint32]$windowProcessId = 0
    [QuickPickCapture]::GetWindowThreadProcessId($h, [ref]$windowProcessId) | Out-Null
    if ($windowProcessId -eq $script:ProcessId -and [QuickPickCapture]::IsWindowVisible($h)) {
      $title = [Text.StringBuilder]::new(512)
      [QuickPickCapture]::GetWindowText($h, $title, 512) | Out-Null
      $class = [Text.StringBuilder]::new(256)
      [QuickPickCapture]::GetClassName($h, $class, 256) | Out-Null
      $rect = New-Object RECT
      [QuickPickCapture]::GetWindowRect($h, [ref]$rect) | Out-Null
      $rows.Add([pscustomobject]@{
        Handle = $h
        Title = $title.ToString()
        Class = $class.ToString()
        Left = $rect.Left
        Top = $rect.Top
        Right = $rect.Right
        Bottom = $rect.Bottom
        Width = [Math]::Max(1, $rect.Right - $rect.Left)
        Height = [Math]::Max(1, $rect.Bottom - $rect.Top)
      })
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:rows
}

function Get-HotkeyWindowHandle([uint32]$ProcessId) {
  $script:ProcessId = $ProcessId
  $script:hotkeyHandle = [IntPtr]::Zero
  [QuickPickCapture]::EnumWindows({
    param($h, $l)
    [uint32]$windowProcessId = 0
    [QuickPickCapture]::GetWindowThreadProcessId($h, [ref]$windowProcessId) | Out-Null
    if ($windowProcessId -eq $script:ProcessId) {
      $class = [Text.StringBuilder]::new(256)
      [QuickPickCapture]::GetClassName($h, $class, 256) | Out-Null
      if ($class.ToString() -eq "global_hotkey_app") {
        $script:hotkeyHandle = $h
        return $false
      }
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:hotkeyHandle
}

function Wait-HotkeyWindowHandle([uint32]$ProcessId) {
  for ($i = 0; $i -lt 20; $i++) {
    $handle = Get-HotkeyWindowHandle $ProcessId
    if ($handle -ne [IntPtr]::Zero) {
      return $handle
    }
    Start-Sleep -Milliseconds 500
  }
  throw "global_hotkey_app window was not found"
}

function Hide-QuickPickWindows([uint32]$ProcessId) {
  $script:ProcessId = $ProcessId
  [QuickPickCapture]::EnumWindows({
    param($h, $l)
    [uint32]$windowProcessId = 0
    [QuickPickCapture]::GetWindowThreadProcessId($h, [ref]$windowProcessId) | Out-Null
    if ($windowProcessId -eq $script:ProcessId) {
      $class = [Text.StringBuilder]::new(256)
      [QuickPickCapture]::GetClassName($h, $class, 256) | Out-Null
      if ($class.ToString() -ne "global_hotkey_app") {
        [QuickPickCapture]::ShowWindow($h, 0) | Out-Null
      }
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
}

function Send-Hotkey([IntPtr]$HotkeyWindow, [int]$Code) {
  $wParam = 0x10000 -bor $Code
  [QuickPickCapture]::PostMessageW($HotkeyWindow, 0x0312, [IntPtr]$wParam, [IntPtr]0x00300000) | Out-Null
}

function Wait-VisibleWindow(
  [uint32]$ProcessId,
  [string]$Title,
  [int]$Attempts = 20
) {
  for ($i = 0; $i -lt $Attempts; $i++) {
    $window = Get-ProcessWindowRows $ProcessId |
      Where-Object { $_.Title -eq $Title -and $_.Class -ne "global_hotkey_app" } |
      Select-Object -First 1
    if ($window) {
      return $window
    }
    Start-Sleep -Milliseconds 300
  }
  throw "QuickPick window did not appear: $Title"
}

function Capture-Window($Window, [string]$Path) {
  [QuickPickCapture]::ShowWindow($Window.Handle, 5) | Out-Null
  [QuickPickCapture]::SetForegroundWindow($Window.Handle) | Out-Null
  [QuickPickCapture]::SetWindowPos(
    $Window.Handle,
    [IntPtr](-1),
    0,
    0,
    0,
    0,
    0x0001 -bor 0x0002 -bor 0x0040
  ) | Out-Null
  Start-Sleep -Milliseconds 700

  $bitmap = New-Object System.Drawing.Bitmap($Window.Width, $Window.Height)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $temporaryPath = $Path + ".capture.tmp"
  try {
    $graphics.CopyFromScreen(
      $Window.Left,
      $Window.Top,
      0,
      0,
      (New-Object System.Drawing.Size($Window.Width, $Window.Height))
    )
    $directory = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    if ($bitmap.Width -gt 1280) {
      $targetWidth = 1280
      $targetHeight = [Math]::Max(1, [int]($bitmap.Height * $targetWidth / $bitmap.Width))
      $resized = New-Object System.Drawing.Bitmap($targetWidth, $targetHeight)
      $resizeGraphics = [System.Drawing.Graphics]::FromImage($resized)
      $resizeGraphics.InterpolationMode = "HighQualityBicubic"
      $resizeGraphics.SmoothingMode = "HighQuality"
      $resizeGraphics.PixelOffsetMode = "HighQuality"
      try {
        $resizeGraphics.DrawImage($bitmap, 0, 0, $targetWidth, $targetHeight)
        $resized.Save($temporaryPath, [System.Drawing.Imaging.ImageFormat]::Png)
      } finally {
        $resizeGraphics.Dispose()
        $resized.Dispose()
      }
    } else {
      $bitmap.Save($temporaryPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
  } finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }
  [IO.File]::Copy($temporaryPath, $Path, $true)
  [IO.File]::Delete($temporaryPath)
}

function Stop-QuickPickProcess {
  Get-Process -Name quickpick -ErrorAction SilentlyContinue |
    Stop-Process -Force
  Start-Sleep -Seconds 1
}

function Set-ScreenshotSettings([string]$Language, [string]$Theme) {
  $directory = Join-Path $env:APPDATA "com.quickpick.desktop"
  $path = Join-Path $directory "settings.json"

  if (Test-Path -LiteralPath $path) {
    $settings = Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json
  } else {
    $settings = [ordered]@{}
  }

  $settings.uiLanguage = $Language
  $settings.themeMode = $Theme
  $settings.windowEffect = "acrylic"
  $settings.panelOpacity = 100
  $settings.autostartEnabled = $false
  $settings.allowClipboardFallback = $false

  New-Item -ItemType Directory -Force -Path $directory | Out-Null
  $json = $settings | ConvertTo-Json -Depth 20
  [IO.File]::WriteAllText($path, $json, [Text.UTF8Encoding]::new($false))
}

function Type-SafeInputText {
  $wsh = New-Object -ComObject WScript.Shell
  $wsh.SendKeys("QuickPick turns selected text and screenshots into AI actions{ENTER}")
  Start-Sleep -Milliseconds 500
}

function Draw-ScreenshotSelection($Window) {
  $startX = $Window.Left + [int]($Window.Width * 0.22)
  $startY = $Window.Top + [int]($Window.Height * 0.24)
  $endX = $Window.Left + [int]($Window.Width * 0.72)
  $endY = $Window.Top + [int]($Window.Height * 0.62)

  [QuickPickCapture]::SetForegroundWindow($Window.Handle) | Out-Null
  Start-Sleep -Milliseconds 400
  [QuickPickCapture]::SetCursorPos($startX, $startY) | Out-Null
  Start-Sleep -Milliseconds 200
  [QuickPickCapture]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150

  for ($step = 1; $step -le 20; $step++) {
    $x = $startX + [int](($endX - $startX) * $step / 20)
    $y = $startY + [int](($endY - $startY) * $step / 20)
    [QuickPickCapture]::SetCursorPos($x, $y) | Out-Null
    Start-Sleep -Milliseconds 30
  }

  [QuickPickCapture]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 800
}

$settingsDirectory = Join-Path $env:APPDATA "com.quickpick.desktop"
$settingsPath = Join-Path $settingsDirectory "settings.json"
$hadExistingSettings = Test-Path -LiteralPath $settingsPath
$backupPath = Join-Path $env:TEMP ("quickpick-readme-settings-" + [Guid]::NewGuid().ToString("N") + ".json")
$wasRunning = [bool](Get-Process -Name quickpick -ErrorAction SilentlyContinue | Select-Object -First 1)

try {
  if ($hadExistingSettings) {
    Copy-Item -LiteralPath $settingsPath -Destination $backupPath -Force
  }
  Stop-QuickPickProcess

  $languageDirectoryMap = @{
    "zh-Hans" = "zh-cn"
    "en" = "en"
  }
  $themes = if ($Themes.Count -gt 0) { $Themes } else { @("light") }
  $languages = @("zh-Hans", "en")

  foreach ($language in $languages) {
    $languageDirectory = $languageDirectoryMap[$language]
    if ($language -eq "en") {
      $settingsTitle = "QuickPick Settings"
      $selectionTitle = "QuickPick Selection"
      $inputTitle = "QuickPick Input Translation"
    } else {
      $selectionTitle = "QuickPick " + [char]0x5212 + [char]0x8BCD
      $inputTitle = "QuickPick " + [char]0x8F93 + [char]0x5165 + [char]0x7FFB + [char]0x8BD1
      $settingsTitle = "QuickPick " + [char]0x8BBE + [char]0x7F6E
    }
    foreach ($theme in $themes) {
      Stop-QuickPickProcess
      Set-ScreenshotSettings $language $theme

      $process = Start-Process -FilePath $QuickPickExe -PassThru
      Start-Sleep -Seconds 7

      $hotkeyWindow = Wait-HotkeyWindowHandle ([uint32]$process.Id)

      Hide-QuickPickWindows ([uint32]$process.Id)
      Start-Sleep -Milliseconds 300
      Send-Hotkey $hotkeyWindow 5
      $window = Wait-VisibleWindow ([uint32]$process.Id) $settingsTitle
      Capture-Window $window (Join-Path $OutputRoot "screenshots\$languageDirectory\settings-$theme.png")

      Hide-QuickPickWindows ([uint32]$process.Id)
      Start-Sleep -Milliseconds 300
      Send-Hotkey $hotkeyWindow 7
      $window = Wait-VisibleWindow ([uint32]$process.Id) $selectionTitle
      Capture-Window $window (Join-Path $OutputRoot "screenshots\$languageDirectory\selection-$theme.png")

      Hide-QuickPickWindows ([uint32]$process.Id)
      Start-Sleep -Milliseconds 300
      Send-Hotkey $hotkeyWindow 9
      $window = Wait-VisibleWindow ([uint32]$process.Id) $inputTitle
      [QuickPickCapture]::SetForegroundWindow($window.Handle) | Out-Null
      Start-Sleep -Milliseconds 800
      Type-SafeInputText
      Capture-Window $window (Join-Path $OutputRoot "screenshots\$languageDirectory\input-$theme.png")

      if ($IncludeScreenshot) {
        $screenshotTitle = "QuickPick " + [char]0x622A + [char]0x56FE
        Hide-QuickPickWindows ([uint32]$process.Id)
        Start-Sleep -Milliseconds 300
        Send-Hotkey $hotkeyWindow 8
        $window = Wait-VisibleWindow ([uint32]$process.Id) $screenshotTitle
        Draw-ScreenshotSelection $window
        Capture-Window $window (Join-Path $OutputRoot "screenshots\$languageDirectory\screenshot-$theme.png")
      }

      Stop-QuickPickProcess
    }
  }
} finally {
  Stop-QuickPickProcess
  if ($hadExistingSettings) {
    Copy-Item -LiteralPath $backupPath -Destination $settingsPath -Force
  } elseif (Test-Path -LiteralPath $settingsPath) {
    Remove-Item -LiteralPath $settingsPath -Force
  }
  if ($wasRunning) {
    Start-Process -FilePath $QuickPickExe
  }
}

Write-Output "README screenshots saved under $OutputRoot\screenshots"
