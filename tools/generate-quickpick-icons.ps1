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

    $platePath = New-RoundedRectPath (& $sx 10) (& $sx 10) (& $sx 108) (& $sx 108) (& $sx 32)
    $gradientBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
        [System.Drawing.RectangleF]::new((& $sx 10), (& $sx 10), (& $sx 108), (& $sx 108)),
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
    & $glow 40 22 44 26 "#E4DCFE" 60
    & $glow 48 24 25 12 "#FFFFFF" 80
    & $glow 36 18 12.5 6 "#FFFFFF" 98
    & $glow 92 104 30 14 "#FFFFFF" 34

    # Soft inner edge light via three layered strokes.
    $edgePath = New-RoundedRectPath (& $sx 11) (& $sx 11) (& $sx 106) (& $sx 106) (& $sx 31)
    foreach ($edge in @(@(3.5, 12), @(2.5, 22), @(1.2, 40))) {
        $edgePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF" $edge[1]), [float](& $sx $edge[0]))
        $graphics.DrawPath($edgePen, $edgePath)
        $edgePen.Dispose()
    }

    $bracketPen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 9))
    $bracketPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $linePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 8))
    $linePen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $linePen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $brackets = @(
        @((52, 38), (38, 38), (38, 52)),
        @((76, 38), (90, 38), (90, 52)),
        @((52, 90), (38, 90), (38, 76)),
        @((76, 90), (90, 90), (90, 76))
    )
    foreach ($bracket in $brackets) {
        $points = $bracket | ForEach-Object { [System.Drawing.PointF]::new((& $sx $_[0]), (& $sx $_[1])) }
        $graphics.DrawLines($bracketPen, [System.Drawing.PointF[]]$points)
    }
    $textLines = @(
        @(50, 53, 74, 53),
        @(50, 65, 80, 65),
        @(50, 77, 68, 77)
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
