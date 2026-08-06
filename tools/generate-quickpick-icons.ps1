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

    # Set E: rounded-square plate, five-stop blue-violet gradient, layered
    # glass highlights, soft inner edge, white brackets + text lines glyph.
    # Geometry is defined in a 128-unit design space.
    $scale = $Size / 128.0
    $sx = { param([double]$value) [float]($value * $scale) }

    $platePath = New-RoundedRectPath (& $sx 2) (& $sx 2) (& $sx 124) (& $sx 124) (& $sx 36)
    $gradientBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        [System.Drawing.RectangleF]::new((& $sx 2), (& $sx 2), (& $sx 124), (& $sx 124)),
        (New-Color "#D2C7FC"),
        (New-Color "#2E4FE0"),
        45.0
    )
    $blend = [System.Drawing.Drawing2D.ColorBlend]::new(5)
    $blend.Colors = @(
        (New-Color "#D2C7FC"),
        (New-Color "#A3B8FB"),
        (New-Color "#64A5FA"),
        (New-Color "#3D79F4"),
        (New-Color "#2E4FE0")
    )
    $blend.Positions = @(0.0, 0.22, 0.5, 0.78, 1.0)
    $gradientBrush.InterpolationColors = $blend
    $graphics.FillPath($gradientBrush, $platePath)

    # Radial glow helper: soft highlights that fade to transparent.
    $glow = {
        param([double]$cx, [double]$cy, [double]$rx, [double]$ry, [string]$hex, [int]$alpha)
        $glowPath = [System.Drawing.Drawing2D.GraphicsPath]::new()
        $glowPath.AddEllipse([float](& $sx ($cx - $rx)), [float](& $sx ($cy - $ry)), [float](& $sx ($rx * 2)), [float](& $sx ($ry * 2)))
        $glowBrush = [System.Drawing.Drawing2D.PathGradientBrush]::new($glowPath)
        $glowBrush.CenterColor = New-Color $hex $alpha
        $glowBrush.SurroundColors = @([System.Drawing.Color]::FromArgb(0, 255, 255, 255))
        $graphics.FillPath($glowBrush, $glowPath)
        $glowBrush.Dispose()
        $glowPath.Dispose()
    }
    # Lavender tint, gloss cloud, specular spot, bottom bounce.
    & $glow 36.4 15.8 50.5 29.8 "#E4DCFE" 60
    & $glow 45.6 18.1 28.7 13.8 "#FFFFFF" 80
    & $glow 32.1 11.2 14.4 6.9 "#FFFFFF" 98
    & $glow 96.2 109.9 34.4 16.1 "#FFFFFF" 34

    # Soft inner edge light via three layered strokes.
    $edgePath = New-RoundedRectPath (& $sx 3.2) (& $sx 3.2) (& $sx 121.6) (& $sx 121.6) (& $sx 35.6)
    foreach ($edge in @(@(4.0, 12), @(2.9, 22), @(1.4, 40))) {
        $edgePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF" $edge[1]), [float](& $sx $edge[0]))
        $graphics.DrawPath($edgePen, $edgePath)
        $edgePen.Dispose()
    }

    $bracketPen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 10.3))
    $bracketPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $linePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 9.2))
    $linePen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $linePen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $brackets = @(
        @((50.2, 34.2), (34.2, 34.2), (34.2, 50.2)),
        @((77.8, 34.2), (93.8, 34.2), (93.8, 50.2)),
        @((50.2, 93.8), (34.2, 93.8), (34.2, 77.8)),
        @((77.8, 93.8), (93.8, 93.8), (93.8, 77.8))
    )
    foreach ($bracket in $brackets) {
        $points = $bracket | ForEach-Object { [System.Drawing.PointF]::new((& $sx $_[0]), (& $sx $_[1])) }
        $graphics.DrawLines($bracketPen, [System.Drawing.PointF[]]$points)
    }
    $textLines = @(
        @(47.9, 51.4, 75.5, 51.4),
        @(47.9, 65.1, 82.4, 65.1),
        @(47.9, 78.9, 68.6, 78.9)
    )
    foreach ($line in $textLines) {
        $graphics.DrawLine($linePen, (& $sx $line[0]), (& $sx $line[1]), (& $sx $line[2]), (& $sx $line[3]))
    }

    $graphics.Dispose()
    $gradientBrush.Dispose()
    $platePath.Dispose()
    $edgePath.Dispose()
    $bracketPen.Dispose()
    $linePen.Dispose()

    $bitmap
}

function New-TrayIconBitmap {
    param([int]$Size)

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    Set-HighQualityGraphics $graphics
    $graphics.Clear([System.Drawing.Color]::Transparent)

    # Tray glyph: white linear "corner brackets + three text lines" (set D).
    # Geometry is defined in a 128-unit design space.
    $scale = $Size / 128.0
    $sx = { param([double]$value) [float]($value * $scale) }

    $bracketPen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 12))
    $bracketPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    $linePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 11))
    $linePen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $linePen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $brackets = @(
        @((40, 11), (11, 11), (11, 40)),
        @((88, 11), (117, 11), (117, 40)),
        @((40, 117), (11, 117), (11, 88)),
        @((88, 117), (117, 117), (117, 88))
    )
    foreach ($bracket in $brackets) {
        $points = $bracket | ForEach-Object { [System.Drawing.PointF]::new((& $sx $_[0]), (& $sx $_[1])) }
        $graphics.DrawLines($bracketPen, [System.Drawing.PointF[]]$points)
    }

    $textLines = @(
        @(36, 42, 85, 42),
        @(36, 66, 97, 66),
        @(36, 90, 73, 90)
    )
    foreach ($line in $textLines) {
        $graphics.DrawLine($linePen, (& $sx $line[0]), (& $sx $line[1]), (& $sx $line[2]), (& $sx $line[3]))
    }

    $graphics.Dispose()
    $bracketPen.Dispose()
    $linePen.Dispose()

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
