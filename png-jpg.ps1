# Get all .png files in the current directory
$pngFiles = Get-ChildItem -Filter *.png

# 'C:\Program Files\ImageMagick-7.1.1-Q16-HDRI\'
# Loop through each .png file and convert to .jpg
foreach ($file in $pngFiles) {
    $outputFileName = [System.IO.Path]::ChangeExtension($file.FullName, "jpg")
    Write-Host "Converting $($file.FullName) to $($outputFileName)"
    
    # Use the Windows built-in 'convert' (Magick) command
    magick convert "$($file.FullName)" "$($outputFileName)"
}

Write-Host "Conversion complete."

# Get all .png files in the current directory
$pngFiles = Get-ChildItem -Filter *.png

# Specify the full path to the magick command
$magickCommand = 'C:\Program Files\ImageMagick-7.1.1-Q16-HDRI\magick.exe'

# Loop through each .png file and convert to .jpg
foreach ($file in $pngFiles) {
    $outputFileName = [System.IO.Path]::ChangeExtension($file.FullName, "jpg")
    Write-Host "Converting $($file.FullName) to $($outputFileName)"
    
    # Use the full path to the 'magick' command
    & $magickCommand convert "$($file.FullName)" "$($outputFileName)"
}

Write-Host "Conversion complete."