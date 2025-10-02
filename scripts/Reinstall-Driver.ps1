#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Reinstalls the HIDraBus driver for development/testing

.DESCRIPTION
    This script automates the process of uninstalling and reinstalling the HIDraBus driver.
    It handles stopping/starting the service, uninstalling old versions, and installing the new driver.

.PARAMETER DriverPath
    Path to the driver .sys file. Defaults to drivers\HIDraBus\x64\Debug\HIDraBus.sys relative to the repo root.

.PARAMETER InfPath  
    Path to the driver .inf file. Defaults to drivers\HIDraBus\x64\Debug\HIDraBus.inf relative to the repo root.

.EXAMPLE
    .\Reinstall-Driver.ps1
    
.EXAMPLE
    .\Reinstall-Driver.ps1 -DriverPath "drivers\HIDraBus\x64\Release\HIDraBus.sys" -InfPath "drivers\HIDraBus\x64\Release\HIDraBus.inf"
#>

param(
    [string]$DriverPath = "..\drivers\HIDraBus\x64\Debug\HIDraBus.sys",
    [string]$InfPath = "..\drivers\HIDraBus\x64\Debug\HIDraBus.inf"
)

$ErrorActionPreference = "Stop"
$DriverName = "HIDraBus"
$ServiceName = "HIDraBus"

# Colors for output
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-ColorOutput {
    param([string]$Message, [string]$Color = $Reset)
    Write-Host "$Color$Message$Reset"
}

function Test-AdminRights {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Stop-DriverService {
    Write-ColorOutput "Stopping driver service..." $Blue
    try {
        $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($service -and $service.Status -eq 'Running') {
            Stop-Service -Name $ServiceName -Force
            Write-ColorOutput "Service stopped successfully" $Green
        } else {
            Write-ColorOutput "Service not running or not found" $Yellow
        }
    } catch {
        Write-ColorOutput "Warning: Could not stop service: $_" $Yellow
    }
}

function Uninstall-Driver {
    Write-ColorOutput "Uninstalling existing driver..." $Blue
    
    # Remove driver package
    try {
        $packages = pnputil /enum-drivers | Select-String -Pattern "Published Name.*oem.*\.inf" -Context 0,10
        foreach ($package in $packages) {
            $context = $package.Context.PostContext
            $driverInfo = $context | Select-String -Pattern "Driver Name.*HIDraBus"
            if ($driverInfo) {
                $oemInf = ($package.Line -split ":")[1].Trim()
                Write-ColorOutput "Found driver package: $oemInf" $Yellow
                pnputil /delete-driver $oemInf /uninstall /force
                Write-ColorOutput "Removed driver package: $oemInf" $Green
            }
        }
    } catch {
        Write-ColorOutput "Warning: Error during driver uninstall: $_" $Yellow
    }
    
    # Delete service if it exists
    try {
        $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($service) {
            sc.exe delete $ServiceName
            Write-ColorOutput "Service deleted" $Green
        }
    } catch {
        Write-ColorOutput "Warning: Could not delete service: $_" $Yellow
    }
}

function Install-Driver {
    Write-ColorOutput "Installing new driver..." $Blue
    
    # Check if files exist
    if (-not (Test-Path $DriverPath)) {
        throw "Driver file not found: $DriverPath"
    }
    if (-not (Test-Path $InfPath)) {
        throw "INF file not found: $InfPath"
    }
    
    # Install driver package
    Write-ColorOutput "Installing driver package..." $Blue
    pnputil /add-driver $InfPath /install
    
    # Create and start service
    Write-ColorOutput "Creating driver service..." $Blue
    $fullDriverPath = Resolve-Path $DriverPath
    sc.exe create $ServiceName binpath= $fullDriverPath type= kernel start= demand
    
    Write-ColorOutput "Starting driver service..." $Blue
    sc.exe start $ServiceName
    
    Write-ColorOutput "Driver installation completed!" $Green
}

function Show-DriverStatus {
    Write-ColorOutput "`nDriver Status:" $Blue
    
    # Check service status
    try {
        $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($service) {
            Write-ColorOutput "Service Status: $($service.Status)" $Green
        } else {
            Write-ColorOutput "Service Status: Not Found" $Red
        }
    } catch {
        Write-ColorOutput "Service Status: Error checking" $Red
    }
    
    # Check for driver packages
    Write-ColorOutput "`nInstalled driver packages:" $Blue
    try {
        $packages = pnputil /enum-drivers | Select-String -Pattern "Published Name.*oem.*\.inf" -Context 0,10
        $found = $false
        foreach ($package in $packages) {
            $context = $package.Context.PostContext
            $driverInfo = $context | Select-String -Pattern "Driver Name.*HIDraBus"
            if ($driverInfo) {
                $oemInf = ($package.Line -split ":")[1].Trim()
                Write-ColorOutput "  $oemInf" $Green
                $found = $true
            }
        }
        if (-not $found) {
            Write-ColorOutput "  No HIDraBus driver packages found" $Yellow
        }
    } catch {
        Write-ColorOutput "  Error checking driver packages" $Red
    }
}

# Main execution
try {
    Write-ColorOutput "HIDraBus Driver Reinstaller" $Blue
    Write-ColorOutput "===========================" $Blue
    
    if (-not (Test-AdminRights)) {
        throw "This script must be run as Administrator"
    }
    
    Write-ColorOutput "Using driver: $DriverPath" $Yellow
    Write-ColorOutput "Using INF: $InfPath" $Yellow
    Write-ColorOutput ""
    
    Stop-DriverService
    Uninstall-Driver
    Start-Sleep -Seconds 2  # Brief pause
    Install-Driver
    
    Show-DriverStatus
    
    Write-ColorOutput "`nDriver reinstallation completed successfully!" $Green
    Write-ColorOutput "You can now test the driver with hidra-broker." $Green
    
} catch {
    Write-ColorOutput "`nError: $_" $Red
    Write-ColorOutput "Driver reinstallation failed!" $Red
    exit 1
}