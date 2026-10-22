# @cypher schema=1 source_sha256=88f4df105be6a0e823cd0110c0dd32fd3a1f66a77f59f3f655eaa5399cf4b02f
# MERGE (self:File {path:"scripts/ui0_bench/capture_windows_inventory.ps1"})
# MERGE (get_inventory_field:Symbol {id:"scripts/ui0_bench/capture_windows_inventory.ps1::Get-InventoryField", kind:"function"})
# MERGE (get_display_inventory:Symbol {id:"scripts/ui0_bench/capture_windows_inventory.ps1::Get-DisplayInventory", kind:"function"})
# MERGE (get_capture_inventory:Symbol {id:"scripts/ui0_bench/capture_windows_inventory.ps1::Get-CaptureInventory", kind:"function"})
# MERGE (self)-[:DEFINES]->(get_inventory_field)
# MERGE (self)-[:DEFINES]->(get_display_inventory)
# MERGE (self)-[:DEFINES]->(get_capture_inventory)
# MERGE (get_display_inventory)-[:CALLS]->(get_inventory_field)
# MERGE (get_capture_inventory)-[:CALLS]->(get_inventory_field)
# MERGE (get_capture_inventory)-[:CALLS]->(get_display_inventory)
# @endcypher

$ErrorActionPreference = 'Stop'

function Get-InventoryField {
    param(
        [Parameter(Mandatory = $true)][scriptblock]$Reader,
        [Parameter(Mandatory = $true)][string]$Method
    )

    try {
        $value = & $Reader
        if ($null -eq $value) {
            throw 'empty_value'
        }
        return @{ status = 'available'; value = $value; method = $Method; reason = $null }
    }
    catch {
        return @{ status = 'unknown'; value = $null; method = $Method; reason = 'collection_failed' }
    }
}

function Get-DisplayInventory {
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        return @{ status = 'unknown'; reason = 'windows_only_api'; method = 'System.Windows.Forms + Win32 display APIs'; displays = @() }
    }

    try {
        Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class Ui0DisplayNative {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct DEVMODE {
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
        public short dmSpecVersion;
        public short dmDriverVersion;
        public short dmSize;
        public short dmDriverExtra;
        public int dmFields;
        public int dmPositionX;
        public int dmPositionY;
        public int dmDisplayOrientation;
        public int dmDisplayFixedOutput;
        public short dmColor;
        public short dmDuplex;
        public short dmYResolution;
        public short dmTTOption;
        public short dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
        public short dmLogPixels;
        public int dmBitsPerPel;
        public int dmPelsWidth;
        public int dmPelsHeight;
        public int dmDisplayFlags;
        public int dmDisplayFrequency;
        public int dmICMMethod;
        public int dmICMIntent;
        public int dmMediaType;
        public int dmDitherType;
        public int dmReserved1;
        public int dmReserved2;
        public int dmPanningWidth;
        public int dmPanningHeight;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int X; public int Y; }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern bool EnumDisplaySettings(string deviceName, int modeNumber, ref DEVMODE mode);

    [DllImport("user32.dll")]
    public static extern IntPtr MonitorFromPoint(POINT point, uint flags);

    [DllImport("Shcore.dll")]
    public static extern int GetDpiForMonitor(IntPtr monitor, int dpiType, out uint dpiX, out uint dpiY);
}
'@ -ErrorAction Stop

        $screens = [System.Windows.Forms.Screen]::AllScreens
        if ($null -eq $screens -or $screens.Count -eq 0) {
            return @{ status = 'unknown'; reason = 'no_active_displays'; method = 'System.Windows.Forms.Screen.AllScreens'; displays = @() }
        }

        $items = @()
        foreach ($screen in $screens) {
            $deviceName = $screen.DeviceName
            $bounds = $screen.Bounds
            $refresh = Get-InventoryField -Method 'user32.EnumDisplaySettings' -Reader {
                $mode = New-Object Ui0DisplayNative+DEVMODE
                $mode.dmSize = [short][Runtime.InteropServices.Marshal]::SizeOf($mode)
                if (-not [Ui0DisplayNative]::EnumDisplaySettings($deviceName, -1, [ref]$mode)) {
                    return $null
                }
                return [int]$mode.dmDisplayFrequency
            }
            $dpi = Get-InventoryField -Method 'Shcore.GetDpiForMonitor' -Reader {
                $point = New-Object Ui0DisplayNative+POINT
                $point.X = [int]($bounds.Left + ($bounds.Width / 2))
                $point.Y = [int]($bounds.Top + ($bounds.Height / 2))
                $monitor = [Ui0DisplayNative]::MonitorFromPoint($point, 2)
                if ($monitor -eq [IntPtr]::Zero) {
                    return $null
                }
                [uint32]$dpiX = 0
                [uint32]$dpiY = 0
                $result = [Ui0DisplayNative]::GetDpiForMonitor($monitor, 0, [ref]$dpiX, [ref]$dpiY)
                if ($result -ne 0) {
                    return $null
                }
                return @{ x = [int]$dpiX; y = [int]$dpiY }
            }
            $items += @{
                device_name = $deviceName
                primary = [bool]$screen.Primary
                resolution = @{ width = [int]$bounds.Width; height = [int]$bounds.Height; method = 'System.Windows.Forms.Screen.Bounds' }
                refresh_hz = $refresh
                dpi = $dpi
            }
        }
        return @{ status = 'available'; reason = $null; method = 'System.Windows.Forms + Win32 display APIs'; displays = $items }
    }
    catch {
        return @{ status = 'unknown'; reason = 'display_api_unavailable'; method = 'System.Windows.Forms + Win32 display APIs'; displays = @() }
    }
}

function Get-CaptureInventory {
    $onWindows = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT
    $os = Get-InventoryField -Method 'Get-CimInstance Win32_OperatingSystem' -Reader {
        $item = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
        @{ caption = [string]$item.Caption; version = [string]$item.Version; build = [string]$item.BuildNumber; architecture = [string]$item.OSArchitecture }
    }
    $cpu = Get-InventoryField -Method 'Get-CimInstance Win32_Processor' -Reader {
        $items = @(Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop | ForEach-Object {
            @{ name = [string]$_.Name; cores = [int]$_.NumberOfCores; logical_processors = [int]$_.NumberOfLogicalProcessors }
        })
        if ($items.Count -eq 0) { return $null }
        return $items
    }
    $memory = Get-InventoryField -Method 'Get-CimInstance Win32_ComputerSystem' -Reader {
        $item = Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction Stop
        @{ physical_bytes = [int64]$item.TotalPhysicalMemory }
    }
    $gpu = Get-InventoryField -Method 'Get-CimInstance Win32_VideoController' -Reader {
        $items = @(Get-CimInstance -ClassName Win32_VideoController -ErrorAction Stop | ForEach-Object {
            @{ name = [string]$_.Name; driver_version = [string]$_.DriverVersion; adapter_ram_cim_reported_bytes = [int64]$_.AdapterRAM }
        })
        if ($items.Count -eq 0) { return $null }
        return $items
    }
    $power = Get-InventoryField -Method 'powercfg.exe /getactivescheme' -Reader {
        if (-not $onWindows) { return $null }
        $lines = @(& powercfg.exe /getactivescheme 2>$null)
        if ($LASTEXITCODE -ne 0 -or $lines.Count -eq 0) { return $null }
        $match = [regex]::Match(($lines -join ' '), '([0-9a-fA-F-]{36})\s+\((.+)\)')
        if (-not $match.Success) { return $null }
        return @{ guid = $match.Groups[1].Value.ToLowerInvariant(); name = $match.Groups[2].Value }
    }

    return @{
        schema = 'star.ui-benchmark-host-inventory'
        version = '0.1.0'
        captured_at = [DateTimeOffset]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ss.fffZ')
        collection_method = @(
            'Get-CimInstance Win32_OperatingSystem',
            'Get-CimInstance Win32_Processor',
            'Get-CimInstance Win32_ComputerSystem',
            'Get-CimInstance Win32_VideoController',
            'System.Windows.Forms + Win32 display APIs',
            'user32.EnumDisplaySettings',
            'Shcore.GetDpiForMonitor',
            'powercfg.exe /getactivescheme'
        )
        os = $os
        cpu = $cpu
        memory = $memory
        gpu = $gpu
        active_power_scheme = $power
        displays = Get-DisplayInventory
    }
}

$inventory = Get-CaptureInventory
$inventory | ConvertTo-Json -Depth 12 -Compress
