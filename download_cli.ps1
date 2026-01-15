##############################################################################
# aster CLI Install Script for Windows PowerShell
#
# This script downloads the latest stable 'aster' CLI binary from GitHub releases
# and installs it to your system.
#
# Supported OS: Windows
# Supported Architectures: x86_64
#
# Usage:
#   Invoke-WebRequest -Uri "https://github.com/block/aster/releases/download/stable/download_cli.ps1" -OutFile "download_cli.ps1"; .\download_cli.ps1
#   Or simply: .\download_cli.ps1
#
# Environment variables:
#   $env:aster_BIN_DIR  - Directory to which aster will be installed (default: $env:USERPROFILE\.local\bin)
#   $env:aster_VERSION  - Optional: specific version to install (e.g., "v1.0.25"). Can be in the format vX.Y.Z, vX.Y.Z-suffix, or X.Y.Z
#   $env:aster_PROVIDER - Optional: provider for aster
#   $env:aster_MODEL    - Optional: model for aster
#   $env:CANARY         - Optional: if set to "true", downloads from canary release instead of stable
#   $env:CONFIGURE      - Optional: if set to "false", disables running aster configure interactively
##############################################################################

# Set error action preference to stop on errors
$ErrorActionPreference = "Stop"

# --- 1) Variables ---
$REPO = "block/aster"
$OUT_FILE = "aster.exe"

# Set default bin directory if not specified
if (-not $env:aster_BIN_DIR) {
    $env:aster_BIN_DIR = Join-Path $env:USERPROFILE ".local\bin"
}

# Determine release type
$RELEASE = if ($env:CANARY -eq "true") { "true" } else { "false" }
$CONFIGURE = if ($env:CONFIGURE -eq "false") { "false" } else { "true" }

# Determine release tag
if ($env:aster_VERSION) {
    # Validate version format
    if ($env:aster_VERSION -notmatch '^v?[0-9]+\.[0-9]+\.[0-9]+(-.*)?$') {
        Write-Error "Invalid version '$env:aster_VERSION'. Expected: semver format vX.Y.Z, vX.Y.Z-suffix, or X.Y.Z"
        exit 1
    }
    # Ensure version starts with 'v'
    $RELEASE_TAG = if ($env:aster_VERSION.StartsWith("v")) { $env:aster_VERSION } else { "v$env:aster_VERSION" }
} else {
    # Use canary or stable based on RELEASE variable
    $RELEASE_TAG = if ($RELEASE -eq "true") { "canary" } else { "stable" }
}

# --- 2) Detect Architecture ---
$ARCH = $env:PROCESSOR_ARCHITECTURE
if ($ARCH -eq "AMD64") {
    $ARCH = "x86_64"
} elseif ($ARCH -eq "ARM64") {
    Write-Error "Windows ARM64 is not currently supported."
    exit 1
} else {
    Write-Error "Unsupported architecture '$ARCH'. Only x86_64 is supported on Windows."
    exit 1
}

# --- 3) Build download URL ---
$FILE = "aster-$ARCH-pc-windows-gnu.zip"
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/$RELEASE_TAG/$FILE"

Write-Host "Downloading $RELEASE_TAG release: $FILE..." -ForegroundColor Green

# --- 4) Download the file ---
try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $FILE -UseBasicParsing
    Write-Host "Download completed successfully." -ForegroundColor Green
} catch {
    Write-Error "Failed to download $DOWNLOAD_URL. Error: $($_.Exception.Message)"
    exit 1
}

# --- 5) Create temporary directory for extraction ---
$TMP_DIR = Join-Path $env:TEMP "aster_install_$(Get-Random)"
try {
    New-Item -ItemType Directory -Path $TMP_DIR -Force | Out-Null
    Write-Host "Created temporary directory: $TMP_DIR" -ForegroundColor Yellow
} catch {
    Write-Error "Could not create temporary extraction directory: $TMP_DIR"
    exit 1
}

# --- 6) Extract the archive ---
Write-Host "Extracting $FILE to temporary directory..." -ForegroundColor Green
try {
    Expand-Archive -Path $FILE -DestinationPath $TMP_DIR -Force
    Write-Host "Extraction completed successfully." -ForegroundColor Green
} catch {
    Write-Error "Failed to extract $FILE. Error: $($_.Exception.Message)"
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
}

# Clean up the downloaded archive
Remove-Item -Path $FILE -Force

# --- 7) Determine extraction directory ---
$EXTRACT_DIR = $TMP_DIR
if (Test-Path (Join-Path $TMP_DIR "aster-package")) {
    Write-Host "Found aster-package subdirectory, using that as extraction directory" -ForegroundColor Yellow
    $EXTRACT_DIR = Join-Path $TMP_DIR "aster-package"
}

# --- 8) Create bin directory if it doesn't exist ---
if (-not (Test-Path $env:aster_BIN_DIR)) {
    Write-Host "Creating directory: $env:aster_BIN_DIR" -ForegroundColor Yellow
    try {
        New-Item -ItemType Directory -Path $env:aster_BIN_DIR -Force | Out-Null
    } catch {
        Write-Error "Could not create directory: $env:aster_BIN_DIR"
        Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }
}

# --- 9) Install aster binary ---
$SOURCE_aster = Join-Path $EXTRACT_DIR "aster.exe"
$DEST_aster = Join-Path $env:aster_BIN_DIR $OUT_FILE

if (Test-Path $SOURCE_aster) {
    Write-Host "Moving aster to $DEST_aster" -ForegroundColor Green
    try {
        # Remove existing file if it exists to avoid conflicts
        if (Test-Path $DEST_aster) {
            Remove-Item -Path $DEST_aster -Force
        }
        Move-Item -Path $SOURCE_aster -Destination $DEST_aster -Force
    } catch {
        Write-Error "Failed to move aster.exe to $DEST_aster. Error: $($_.Exception.Message)"
        Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }
} else {
    Write-Error "aster.exe not found in extracted files"
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
}

# --- 10) Copy Windows runtime DLLs if they exist ---
$DLL_FILES = Get-ChildItem -Path $EXTRACT_DIR -Filter "*.dll" -ErrorAction SilentlyContinue
foreach ($dll in $DLL_FILES) {
    $DEST_DLL = Join-Path $env:aster_BIN_DIR $dll.Name
    Write-Host "Moving Windows runtime DLL: $($dll.Name)" -ForegroundColor Green
    try {
        # Remove existing file if it exists to avoid conflicts
        if (Test-Path $DEST_DLL) {
            Remove-Item -Path $DEST_DLL -Force
        }
        Move-Item -Path $dll.FullName -Destination $DEST_DLL -Force
    } catch {
        Write-Warning "Failed to move $($dll.Name): $($_.Exception.Message)"
    }
}

# --- 11) Clean up temporary directory ---
try {
    Remove-Item -Path $TMP_DIR -Recurse -Force
    Write-Host "Cleaned up temporary directory." -ForegroundColor Yellow
} catch {
    Write-Warning "Could not clean up temporary directory: $TMP_DIR"
}

# --- 12) Configure aster (Optional) ---
if ($CONFIGURE -eq "true") {
    Write-Host ""
    Write-Host "Configuring aster" -ForegroundColor Green
    Write-Host ""
    try {
        & $DEST_aster configure
    } catch {
        Write-Warning "Failed to run aster configure. You may need to run it manually later."
    }
} else {
    Write-Host "Skipping 'aster configure', you may need to run this manually later" -ForegroundColor Yellow
}

# --- 13) Check PATH and give instructions if needed ---
$CURRENT_PATH = $env:PATH
if ($CURRENT_PATH -notlike "*$env:aster_BIN_DIR*") {
    Write-Host ""
    Write-Host "Warning: aster installed, but $env:aster_BIN_DIR is not in your PATH." -ForegroundColor Yellow
    Write-Host "To add it to your PATH permanently, run the following command as Administrator:" -ForegroundColor Yellow
    Write-Host "    [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$env:aster_BIN_DIR', 'Machine')" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Or add it to your user PATH (no admin required):" -ForegroundColor Yellow
    Write-Host "    [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$env:aster_BIN_DIR', 'User')" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "For this session only, you can run:" -ForegroundColor Yellow
    Write-Host "    `$env:PATH += ';$env:aster_BIN_DIR'" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "aster CLI installation completed successfully!" -ForegroundColor Green
Write-Host "aster is installed at: $DEST_aster" -ForegroundColor Green
