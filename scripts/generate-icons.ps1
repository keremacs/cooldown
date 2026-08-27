# Generates Cooldown app icons (PNG + ICO) for Tauri bundle and system tray.
Add-Type -AssemblyName System.Drawing

function Draw-CooldownIcon {
    param([int]$Size)
    $bmp = New-Object System.Drawing.Bitmap $Size, $Size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear([System.Drawing.Color]::FromArgb(0, 0, 0, 0))

    $margin = [math]::Max(2, [int]($Size * 0.06))
    $rect = New-Object System.Drawing.Rectangle $margin, $margin, ($Size - 2 * $margin), ($Size - 2 * $margin)

    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $rect,
        [System.Drawing.Color]::FromArgb(255, 79, 70, 229),
        [System.Drawing.Color]::FromArgb(255, 99, 102, 241),
        45
    )
    $g.FillEllipse($brush, $rect)
    $brush.Dispose()

    $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(200, 255, 255, 255)), ([math]::Max(1, $Size / 32))
    $cx = $Size / 2
    $cy = $Size / 2
    $r = $Size * 0.22
    # Snowflake / cool symbol — three arcs
    for ($i = 0; $i -lt 3; $i++) {
        $angle = $i * 120 * [Math]::PI / 180
        $x1 = $cx + $r * [Math]::Cos($angle)
        $y1 = $cy + $r * [Math]::Sin($angle)
        $x2 = $cx - $r * [Math]::Cos($angle)
        $y2 = $cy - $r * [Math]::Sin($angle)
        $g.DrawLine($pen, $x1, $y1, $x2, $y2)
    }
    $g.FillEllipse([System.Drawing.Brushes]::White, ($cx - $r / 3), ($cy - $r / 3), ($r * 2 / 3), ($r * 2 / 3))
    $pen.Dispose()
    $g.Dispose()
    return $bmp
}

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$iconDir = Join-Path $root "src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null

$sizes = @{
    "32x32.png"       = 32
    "128x128.png"     = 128
    "128x128@2x.png"  = 256
    "icon.png"        = 512
    "tray-icon.png"   = 32
}

foreach ($entry in $sizes.GetEnumerator()) {
    $bmp = Draw-CooldownIcon -Size $entry.Value
    $path = Join-Path $iconDir $entry.Key
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "Created $path"
}

Write-Host "Created $icoPath (run node scripts/make-ico.mjs for valid ICO)"
Write-Host "Done."
