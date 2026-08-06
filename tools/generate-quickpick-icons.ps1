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

    # Tray glyph: white linear "corner brackets + three text lines" (set D).
    # Geometry is defined in a 128-unit design space.
    $scale = $Size / 128.0
    $sx = { param([double]$value) [float]($value * $scale) }

    $bracketPen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 12))
    $bracketPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $bracketPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    $linePen = [System.Drawing.Pen]::new((New-Color "#FFFFFF"), [float](& $sx 10))
    $linePen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $linePen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $brackets = @(
        @((48, 18), (18, 18), (18, 48)),
        @((80, 18), (110, 18), (110, 48)),
        @((48, 110), (18, 110), (18, 80)),
        @((80, 110), (110, 110), (110, 80))
    )
    foreach ($bracket in $brackets) {
        $points = $bracket | ForEach-Object { [System.Drawing.PointF]::new((& $sx $_[0]), (& $sx $_[1])) }
        $graphics.DrawLines($bracketPen, [System.Drawing.PointF[]]$points)
    }

    $textLines = @(
        @(38, 50, 74, 50),
        @(38, 64, 86, 64),
        @(38, 78, 66, 78)
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
