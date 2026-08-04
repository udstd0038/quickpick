Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$projectRoot = Split-Path -Parent $PSScriptRoot
$iconDir = Join-Path $projectRoot "src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null

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

function New-RoundedRectPath {
    param(
        [float]$X,
        [float]$Y,
        [float]$Width,
        [float]$Height,
        [float]$Radius
    )

    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $diameter = $Radius * 2
    $path.AddArc($X, $Y, $diameter, $diameter, 180, 90)
    $path.AddArc($X + $Width - $diameter, $Y, $diameter, $diameter, 270, 90)
    $path.AddArc($X + $Width - $diameter, $Y + $Height - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($X, $Y + $Height - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    $path
}

function Set-HighQualityGraphics {
    param([System.Drawing.Graphics]$Graphics)

    $Graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $Graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $Graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $Graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
}

function New-AppIconBitmap {
    param([int]$Size)

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    Set-HighQualityGraphics $graphics
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $scale = $Size / 1024.0
    $sx = { param([double]$value) [float]($value * $scale) }
    $rect = [System.Drawing.RectangleF]::new((& $sx 88), (& $sx 72), (& $sx 848), (& $sx 880))
    $bgPath = New-RoundedRectPath $rect.X $rect.Y $rect.Width $rect.Height (& $sx 230)

    $bgBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $rect,
        (New-Color "#FFFFFF"),
        (New-Color "#DDEEFF"),
        45.0
    )
    $graphics.FillPath($bgBrush, $bgPath)

    $topGlow = [System.Drawing.SolidBrush]::new((New-Color "#FFFFFF" 188))
    $graphics.FillEllipse($topGlow, (& $sx 156), (& $sx 96), (& $sx 610), (& $sx 270))
    $bottomGlow = [System.Drawing.SolidBrush]::new((New-Color "#8CCBFF" 42))
    $graphics.FillEllipse($bottomGlow, (& $sx 500), (& $sx 710), (& $sx 300), (& $sx 150))
    $borderPen = [System.Drawing.Pen]::new((New-Color "#3A80ED" 44), [float](& $sx 12))
    $graphics.DrawPath($borderPen, $bgPath)
    $innerPath = New-RoundedRectPath (& $sx 118) (& $sx 104) (& $sx 788) (& $sx 814) (& $sx 202)
    $innerPen = [System.Drawing.Pen]::new((New-Color "#FFFFFF" 218), [float](& $sx 5))
    $graphics.DrawPath($innerPen, $innerPath)

    $arcBox = [System.Drawing.RectangleF]::new((& $sx 270), (& $sx 272), (& $sx 492), (& $sx 492))
    $glowPen = [System.Drawing.Pen]::new((New-Color "#7DB7FF" 74), [float](& $sx 124))
    $glowPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $glowPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($glowPen, $arcBox, 145, 300)

    $mainPen = [System.Drawing.Pen]::new((New-Color "#2F7DF6" 242), [float](& $sx 88))
    $mainPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $mainPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($mainPen, $arcBox, 145, 300)

    $tailPen = [System.Drawing.Pen]::new((New-Color "#2F7DF6" 242), [float](& $sx 88))
    $tailPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $tailPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawLine($tailPen, (& $sx 626), (& $sx 738), (& $sx 778), (& $sx 810))

    $graphics.Dispose()
    $bgBrush.Dispose()
    $topGlow.Dispose()
    $bottomGlow.Dispose()
    $borderPen.Dispose()
    $innerPen.Dispose()
    $glowPen.Dispose()
    $mainPen.Dispose()
    $tailPen.Dispose()
    $bgPath.Dispose()
    $innerPath.Dispose()

    $bitmap
}

function New-TrayIconBitmap {
    param([int]$Size)

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    Set-HighQualityGraphics $graphics
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $scale = $Size / 64.0
    $sx = { param([double]$value) [float]($value * $scale) }
    $circle = [System.Drawing.RectangleF]::new((& $sx 4), (& $sx 4), (& $sx 56), (& $sx 56))
    $bgBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        $circle,
        (New-Color "#3A3A3A"),
        (New-Color "#222222"),
        45.0
    )
    $borderColor = New-Color "#F1F1F1" 38
    $glowColor = New-Color "#FFFFFF" 26
    $markColor = New-Color "#D8D8D8" 248
    $markShadowColor = New-Color "#000000" 72
    $topTextureColor = New-Color "#FFFFFF" 31
    $bottomTextureColor = New-Color "#000000" 46
    $highlightColor = New-Color "#FFFFFF" 72
    $graphics.FillEllipse($bgBrush, $circle)
    $borderPen = [System.Drawing.Pen]::new($borderColor, [float](& $sx 2.6))
    $graphics.DrawEllipse($borderPen, $circle)
    $topTextureBrush = [System.Drawing.SolidBrush]::new($topTextureColor)
    $graphics.FillEllipse($topTextureBrush, (& $sx 13), (& $sx 8), (& $sx 34), (& $sx 16))
    $bottomTextureBrush = [System.Drawing.SolidBrush]::new($bottomTextureColor)
    $graphics.FillEllipse($bottomTextureBrush, (& $sx 28), (& $sx 49), (& $sx 30), (& $sx 11))

    $arcBox = [System.Drawing.RectangleF]::new((& $sx 15), (& $sx 13), (& $sx 38), (& $sx 38))
    $shadowArcBox = [System.Drawing.RectangleF]::new((& $sx 16), (& $sx 15), (& $sx 38), (& $sx 38))
    $shadowPen = [System.Drawing.Pen]::new($markShadowColor, [float](& $sx 8.2))
    $shadowPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $shadowPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($shadowPen, $shadowArcBox, 145, 300)
    $graphics.DrawLine($shadowPen, (& $sx 44.7), (& $sx 50.2), (& $sx 56.4), (& $sx 56.8))

    $glowPen = [System.Drawing.Pen]::new($glowColor, [float](& $sx 8.5))
    $glowPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $glowPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($glowPen, $arcBox, 145, 300)

    $markPen = [System.Drawing.Pen]::new($markColor, [float](& $sx 6.2))
    $markPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $markPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($markPen, $arcBox, 145, 300)
    $graphics.DrawLine($markPen, (& $sx 43.5), (& $sx 48.6), (& $sx 55.2), (& $sx 55.2))

    $highlightPen = [System.Drawing.Pen]::new($highlightColor, [float](& $sx 1.8))
    $highlightPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $highlightPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $graphics.DrawArc($highlightPen, $arcBox, 205, 112)

    $graphics.Dispose()
    $bgBrush.Dispose()
    $borderPen.Dispose()
    $topTextureBrush.Dispose()
    $bottomTextureBrush.Dispose()
    $shadowPen.Dispose()
    $glowPen.Dispose()
    $markPen.Dispose()
    $highlightPen.Dispose()

    $bitmap
}

function Get-PngBytes {
    param([System.Drawing.Bitmap]$Bitmap)

    $stream = [System.IO.MemoryStream]::new()
    $Bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $stream.ToArray()
    $stream.Dispose()
    ,$bytes
}

function Write-IcoFile {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int[]]$Sizes
    )

    $images = @()
    foreach ($size in $Sizes) {
        $bitmap = New-AppIconBitmap $size
        $images += [PSCustomObject]@{
            Size = $size
            Bytes = Get-PngBytes $bitmap
        }
        $bitmap.Dispose()
    }

    $fileStream = [System.IO.File]::Create($Path)
    $writer = [System.IO.BinaryWriter]::new($fileStream)
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$images.Count)

    $offset = 6 + (16 * $images.Count)
    foreach ($image in $images) {
        $writer.Write([byte]($(if ($image.Size -ge 256) { 0 } else { $image.Size })))
        $writer.Write([byte]($(if ($image.Size -ge 256) { 0 } else { $image.Size })))
        $writer.Write([byte]0)
        $writer.Write([byte]0)
        $writer.Write([UInt16]1)
        $writer.Write([UInt16]32)
        $writer.Write([UInt32]$image.Bytes.Length)
        $writer.Write([UInt32]$offset)
        $offset += $image.Bytes.Length
    }

    foreach ($image in $images) {
        $writer.Write($image.Bytes)
    }

    $writer.Dispose()
    $fileStream.Dispose()
}

$appBitmap = New-AppIconBitmap 512
$appBitmap.Save((Join-Path $iconDir "icon.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$appBitmap.Dispose()

$trayBitmap = New-TrayIconBitmap 64
$trayBitmap.Save((Join-Path $iconDir "tray-icon.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$trayBitmap.Dispose()

Write-IcoFile (Join-Path $iconDir "icon.ico") @(16, 20, 24, 32, 40, 48, 64, 128, 256)

Write-Host "QuickPick icons generated in $iconDir"
