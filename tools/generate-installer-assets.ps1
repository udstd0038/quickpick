Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$projectRoot = Split-Path -Parent $PSScriptRoot
$installerDir = Join-Path $projectRoot "apps\desktop\src-tauri\installer"
$iconPath = Join-Path $projectRoot "apps\desktop\src-tauri\icons\icon.png"

New-Item -ItemType Directory -Force -Path $installerDir | Out-Null

function New-Color {
    param(
        [Parameter(Mandatory = $true)][string]$Hex,
        [int]$Alpha = 255
    )

    $clean = $Hex.TrimStart("#")
    [System.Drawing.Color]::FromArgb(
        $Alpha,
        [Convert]::ToInt32($clean.Substring(0, 2), 16),
        [Convert]::ToInt32($clean.Substring(2, 2), 16),
        [Convert]::ToInt32($clean.Substring(4, 2), 16)
    )
}

function Set-HighQualityGraphics {
    param([System.Drawing.Graphics]$Graphics)

    $Graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $Graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $Graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $Graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $Graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
}

function Draw-ScaledIcon {
    param(
        [Parameter(Mandatory = $true)][System.Drawing.Graphics]$Graphics,
        [Parameter(Mandatory = $true)][System.Drawing.Image]$Icon,
        [float]$X,
        [float]$Y,
        [float]$Width,
        [float]$Height
    )

    $destination = [System.Drawing.Rectangle]::new(
        [int][Math]::Round($X),
        [int][Math]::Round($Y),
        [int][Math]::Round($Width),
        [int][Math]::Round($Height)
    )
    $Graphics.DrawImage($Icon, $destination, 0, 0, $Icon.Width, $Icon.Height, [System.Drawing.GraphicsUnit]::Pixel)
}

function New-InstallerHeaderBitmap {
    $bitmap = [System.Drawing.Bitmap]::new(150, 57, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    Set-HighQualityGraphics $graphics

    $bounds = [System.Drawing.RectangleF]::new(0, 0, 150, 57)
    $background = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $bounds,
        (New-Color "#F8FBFF"),
        (New-Color "#E7EEFF"),
        0.0
    )
    $graphics.FillRectangle($background, $bounds)

    $icon = [System.Drawing.Image]::FromFile($iconPath)
    Draw-ScaledIcon $graphics $icon 9 8 40 40

    $titleFont = [System.Drawing.Font]::new("Segoe UI", 14.0, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $subtitleFont = [System.Drawing.Font]::new("Segoe UI", 8.0, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $titleBrush = [System.Drawing.SolidBrush]::new((New-Color "#173B8C"))
    $subtitleBrush = [System.Drawing.SolidBrush]::new((New-Color "#5673A8"))

    $graphics.DrawString("QuickPick", $titleFont, $titleBrush, 57.0, 10.0)
    $graphics.DrawString("AI Selection & Translation", $subtitleFont, $subtitleBrush, 57.0, 32.0)

    $icon.Dispose()
    $titleFont.Dispose()
    $subtitleFont.Dispose()
    $titleBrush.Dispose()
    $subtitleBrush.Dispose()
    $background.Dispose()
    $graphics.Dispose()

    $bitmap
}

function New-InstallerSidebarBitmap {
    $bitmap = [System.Drawing.Bitmap]::new(164, 314, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    Set-HighQualityGraphics $graphics

    $bounds = [System.Drawing.RectangleF]::new(0, 0, 164, 314)
    $background = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $bounds,
        (New-Color "#2E4FE0"),
        (New-Color "#64A5FA"),
        90.0
    )
    $blend = [System.Drawing.Drawing2D.ColorBlend]::new(5)
    $blend.Colors = @(
        (New-Color "#2E4FE0"),
        (New-Color "#3D79F4"),
        (New-Color "#64A5FA"),
        (New-Color "#A3B8FB"),
        (New-Color "#D2C7FC")
    )
    $blend.Positions = @(0.0, 0.28, 0.55, 0.8, 1.0)
    $background.InterpolationColors = $blend
    $graphics.FillRectangle($background, $bounds)

    $icon = [System.Drawing.Image]::FromFile($iconPath)
    Draw-ScaledIcon $graphics $icon 34 42 96 96

    $titleFont = [System.Drawing.Font]::new("Segoe UI", 17.0, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $subtitleFont = [System.Drawing.Font]::new("Segoe UI", 8.5, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $titleBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::White)
    $subtitleBrush = [System.Drawing.SolidBrush]::new((New-Color "#FFFFFF" 205))

    $titleFormat = [System.Drawing.StringFormat]::new()
    $titleFormat.Alignment = [System.Drawing.StringAlignment]::Center
    $titleFormat.LineAlignment = [System.Drawing.StringAlignment]::Center

    $titleRect = [System.Drawing.RectangleF]::new(10, 156, 144, 32)
    $subtitleRect = [System.Drawing.RectangleF]::new(14, 190, 136, 36)

    $graphics.DrawString("QuickPick", $titleFont, $titleBrush, $titleRect, $titleFormat)
    $graphics.DrawString("AI Selection & Translation", $subtitleFont, $subtitleBrush, $subtitleRect, $titleFormat)

    $icon.Dispose()
    $titleFont.Dispose()
    $subtitleFont.Dispose()
    $titleBrush.Dispose()
    $subtitleBrush.Dispose()
    $titleFormat.Dispose()
    $background.Dispose()
    $graphics.Dispose()

    $bitmap
}

$header = New-InstallerHeaderBitmap
$header.Save((Join-Path $installerDir "installer-header.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Dispose()

$sidebar = New-InstallerSidebarBitmap
$sidebar.Save((Join-Path $installerDir "installer-sidebar.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Dispose()

Write-Host "QuickPick NSIS installer assets generated in $installerDir"
