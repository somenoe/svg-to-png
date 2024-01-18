# Image Conversion Project

This project provides scripts to convert SVG files to PNG and subsequently convert the generated PNG files to JPG.

## to-png.py

### Usage

Make sure you have Python installed on your system.

1. Install the required Python packages:

```bash
pip install cairosvg
```

2. Run the `to-png.py` script:

```bash
python to-png.py
```

This script converts a range of SVG files (child_5.svg to child_30.svg) to PNG format using the CairoSVG library.

## png-jpg.ps1

### Prerequisites

- [ImageMagick](https://imagemagick.org/) should be installed on your system. I use choco command:
```powershell
choco install imagemagick.app
```

### Usage

1. Open a PowerShell terminal.

2. Run the `png-jpg.ps1` script:

```powershell
./png-jpg.ps1
```

This script performs two conversions:
   - Converts all .png files in the current directory to .jpg using the Windows built-in 'convert' (Magick) command.
   - Converts all .png files to .jpg using the ImageMagick command-line tool.

### Note

Ensure that the ImageMagick installation path is correctly set in the script.