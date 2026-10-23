Add-Type -AssemblyName System.Drawing

$srcPath = "c:\Users\sadia\TIDE\TIDE.png"
$src = [System.Drawing.Bitmap]::FromFile($srcPath)

foreach ($gridSize in @(32, 40, 48)) {
    $pixelBmp = New-Object System.Drawing.Bitmap($gridSize, $gridSize)
    $g = [System.Drawing.Graphics]::FromImage($pixelBmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.DrawImage($src, 0, 0, $gridSize, $gridSize)
    $g.Dispose()

    $scale = [math]::Floor(384 / $gridSize)
    $outSize = $gridSize * $scale
    $scaledBmp = New-Object System.Drawing.Bitmap($outSize, $outSize)
    $g2 = [System.Drawing.Graphics]::FromImage($scaledBmp)
    $g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $g2.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $g2.DrawImage($pixelBmp, 0, 0, $outSize, $outSize)
    $g2.Dispose()

    $dst = "c:\Users\sadia\TIDE\web\public\tide-pixel-$gridSize.png"
    $scaledBmp.Save($dst, [System.Drawing.Imaging.ImageFormat]::Png)
    Write-Host "Saved $dst"

    $pixelBmp.Dispose()
    $scaledBmp.Dispose()
}

$src.Dispose()
