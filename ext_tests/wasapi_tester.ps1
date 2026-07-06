# ==============================================================================
# DISCLOSURE, SEPARATION & LICENSING NOTICE
# ==============================================================================
# This file was created with assistance from AI tools (GLM-4.7). 
# 
# SEPARATION OF CODEBASE: This file is NOT part of the primary human-authored 
# codebase of this project. It is written in a separate programming language 
# and exists solely within an isolated testing environment to serve as an 
# external verification source to ensure cpvc functions correctly.
#
# LICENSE EXCLUSION: This file is strictly EXCLUDED from the GNU General Public 
# License (GPL) governing the rest of this repository. 
#
# PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.
# ==============================================================================

param(
    [switch]$ListAll,
    [switch]$ListOut,
    [switch]$ListIn,
    [switch]$Active,
    [switch]$Disabled,
    [switch]$Disconnected,
    [switch]$NF, # No Formatting flag
    [string]$F,  # Custom Format flag (S=Status, N=Name, I=Id)
    [string]$Id,
    [string]$Name,
    
    # Volume/Mute controls
    [float]$SetVolume,
    [switch]$GetVolume,
    [switch]$Mute,
    [switch]$Unmute,
    [switch]$GetMute,
    [string]$DeviceName,
    
    [ValidateSet("Name", "Type", "Id", "State")]
    [string]$Out
)

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

[Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioEndpointVolume
{
    int RegisterControlChangeNotify(IntPtr pNotify);           
    int UnregisterControlChangeNotify(IntPtr pNotify);         
    int GetChannelCount(out uint pnChannelCount);              
    int SetMasterVolumeLevel(float fLevelDB, IntPtr pguidEventContext);      
    int SetMasterVolumeLevelScalar(float fLevel, IntPtr pguidEventContext);  
    int GetMasterVolumeLevel(out float pfLevelDB);             
    int GetMasterVolumeLevelScalar(out float pfLevel);         
    int SetChannelVolumeLevel(uint nChannel, float fLevelDB, IntPtr pguidEventContext);     
    int SetChannelVolumeLevelScalar(uint nChannel, float fLevel, IntPtr pguidEventContext); 
    int GetChannelVolumeLevel(uint nChannel, out float pfLevelDB);     
    int GetChannelVolumeLevelScalar(uint nChannel, out float pfLevel); 
    int SetMute([MarshalAs(UnmanagedType.Bool)] bool bMute, IntPtr pguidEventContext); 
    int GetMute(out bool pbMute);                              
    int QueryHardwareSupport(out uint pdwHardwareSupportMask); 
    int GetVolumeRange(out float pflVolumeMindB, out float pflVolumeMaxdB, out float pflVolumeIncrementdB); 
    int GetVolumeStepInfo(out uint pnStep, out uint pnStepCount); 
    int VolumeStepUp(IntPtr pguidEventContext);                
    int VolumeStepDown(IntPtr pguidEventContext);              
}

[Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevice
{
    int Activate(ref Guid id, int clsCtx, IntPtr activationParams, out IntPtr ppInterface);
    int OpenPropertyStore(int access, out IntPtr props);
    int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
    int GetState(out int state);
}

[Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceCollection
{
    int GetCount(out uint count);
    int Item(uint index, out IMMDevice device);
}

[StructLayout(LayoutKind.Sequential)]
struct PROPERTYKEY
{
    public Guid fmtid;
    public int pid;
}

// Using IntPtr for PropVariant to bypass CLR marshaling bug (E_NOINTERFACE)
[Guid("8863808B-CA88-4744-9F89-FEA49B2AB03E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPropertyStore
{
    int GetCount(out uint count);
    int GetAt(uint index, out PROPERTYKEY key);
    int GetValue(ref PROPERTYKEY key, IntPtr pv);
    int SetValue(ref PROPERTYKEY key, IntPtr pv);
    int Commit();
}

[Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceEnumerator
{
    int EnumAudioEndpoints(int dataFlow, int stateMask, out IMMDeviceCollection collection);
    int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice endpoint);
}

[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")] 
class MMDeviceEnumeratorComObject { }

internal static class NativeMethods
{
    [DllImport("Ole32.dll", PreserveSig = false)]
    public static extern void PropVariantClear(IntPtr pvar);
}

public class AudioDevice
{
    public string Id { get; set; }
    public string Name { get; set; }
    public string Type { get; set; }
    public int State { get; set; }
}

public class Audio
{
    static readonly PROPERTYKEY PKEY_Device_FriendlyName = new PROPERTYKEY {
        fmtid = new Guid("a45c254e-df1c-4efd-8020-67d146a850e0"),
        pid = 14
    };

    private static string ReadDeviceNameFromPropertyStore(IPropertyStore props)
    {
        PROPERTYKEY key = PKEY_Device_FriendlyName;
        // Allocate 16 bytes for PROPVARIANT (standard size for the union)
        IntPtr pvPtr = Marshal.AllocCoTaskMem(16);
        try
        {
            Marshal.ThrowExceptionForHR(props.GetValue(ref key, pvPtr));
            
            // VT_LPWSTR (Unicode string pointer) is type 31
            short vt = Marshal.ReadInt16(pvPtr);
            if (vt == 31)
            {
                // The string pointer is at offset 8 in the PROPVARIANT struct
                IntPtr pwszVal = Marshal.ReadIntPtr(pvPtr, 8);
                return Marshal.PtrToStringUni(pwszVal);
            }
            return null;
        }
        finally
        {
            NativeMethods.PropVariantClear(pvPtr);
            Marshal.FreeCoTaskMem(pvPtr);
        }
    }

    private static string ReadDeviceName(IMMDevice device)
    {
        IntPtr propStorePtr;
        Marshal.ThrowExceptionForHR(device.OpenPropertyStore(0, out propStorePtr));
        try
        {
            IPropertyStore props = (IPropertyStore)Marshal.GetObjectForIUnknown(propStorePtr);
            try
            {
                string name = ReadDeviceNameFromPropertyStore(props);
                return string.IsNullOrEmpty(name) ? "Unknown Device" : name;
            }
            finally { Marshal.ReleaseComObject(props); }
        }
        finally { Marshal.Release(propStorePtr); }
    }

    private static void ExecuteVolumeAction(string deviceName, Action<IAudioEndpointVolume> action)
    {
        IMMDeviceEnumerator enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
        IMMDevice device = null;

        try
        {
            if (string.IsNullOrEmpty(deviceName))
            {
                Marshal.ThrowExceptionForHR(enumerator.GetDefaultAudioEndpoint(0, 0, out device));
            }
            else
            {
                IMMDeviceCollection collection;
                Marshal.ThrowExceptionForHR(enumerator.EnumAudioEndpoints(0, 7, out collection));
                try
                {
                    uint count;
                    Marshal.ThrowExceptionForHR(collection.GetCount(out count));
                    bool found = false;
                    for (uint i = 0; i < count; i++)
                    {
                        IMMDevice d;
                        Marshal.ThrowExceptionForHR(collection.Item(i, out d));
                        try
                        {
                            string name = ReadDeviceName(d);
                            if (string.Equals(name, deviceName, StringComparison.OrdinalIgnoreCase))
                            {
                                device = d;
                                found = true;
                                break;
                            }
                        }
                        finally 
                        { 
                            if (!found) Marshal.ReleaseComObject(d); 
                        }
                    }
                    if (!found) throw new Exception("Device not found: " + deviceName);
                }
                finally { Marshal.ReleaseComObject(collection); }
            }

            IntPtr volumePtr;
            Guid epvid = typeof(IAudioEndpointVolume).GUID;
            Marshal.ThrowExceptionForHR(device.Activate(ref epvid, 23, IntPtr.Zero, out volumePtr));
            try
            {
                IAudioEndpointVolume volume = (IAudioEndpointVolume)Marshal.GetObjectForIUnknown(volumePtr);
                try
                {
                    action(volume);
                }
                finally { Marshal.ReleaseComObject(volume); }
            }
            finally { Marshal.Release(volumePtr); }
        }
        finally
        {
            if (device != null) Marshal.ReleaseComObject(device);
            Marshal.ReleaseComObject(enumerator);
        }
    }

    public static List<AudioDevice> GetDevices(int dataFlow = -1)
    {
        var devices = new List<AudioDevice>();
        IMMDeviceEnumerator enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
        
        int[] flowsToEnum = dataFlow == -1 ? new int[] { 0, 1 } : new int[] { dataFlow };
        string[] flowNames = { "Playback (Output)", "Recording (Input)" };
        
        try
        {
            for (int f = 0; f < flowsToEnum.Length; f++)
            {
                int flow = flowsToEnum[f];
                IMMDeviceCollection collection;
                // stateMask = 7 gets all devices (Active, Disabled, Disconnected)
                Marshal.ThrowExceptionForHR(enumerator.EnumAudioEndpoints(flow, 7, out collection));
                try
                {
                    uint count;
                    Marshal.ThrowExceptionForHR(collection.GetCount(out count));
                    
                    for (uint i = 0; i < count; i++)
                    {
                        IMMDevice dev;
                        Marshal.ThrowExceptionForHR(collection.Item(i, out dev));
                        try
                        {
                            string deviceId;
                            Marshal.ThrowExceptionForHR(dev.GetId(out deviceId));
                            
                            string name = ReadDeviceName(dev);
                            
                            int state;
                            Marshal.ThrowExceptionForHR(dev.GetState(out state));
                            
                            devices.Add(new AudioDevice
                            {
                                Id = deviceId,
                                Name = name,
                                Type = flowNames[f],
                                State = state
                            });
                        }
                        finally { Marshal.ReleaseComObject(dev); }
                    }
                }
                finally { Marshal.ReleaseComObject(collection); }
            }
        }
        finally { Marshal.ReleaseComObject(enumerator); }
        
        return devices;
    }

    public static float GetVolume(string deviceName = null)
    {
        float v = -1f;
        ExecuteVolumeAction(deviceName, (vol) => {
            Marshal.ThrowExceptionForHR(vol.GetMasterVolumeLevelScalar(out v));
        });
        return v;
    }

    public static void SetVolume(string deviceName, float level)
    {
        // Clamp level between 0 and 1
        level = Math.Max(0f, Math.Min(1f, level));
        ExecuteVolumeAction(deviceName, (vol) => {
            Marshal.ThrowExceptionForHR(vol.SetMasterVolumeLevelScalar(level, IntPtr.Zero));
        });
    }

    public static bool GetMute(string deviceName = null)
    {
        bool mute = false;
        ExecuteVolumeAction(deviceName, (vol) => {
            Marshal.ThrowExceptionForHR(vol.GetMute(out mute));
        });
        return mute;
    }

    public static void SetMute(string deviceName, bool muted)
    {
        ExecuteVolumeAction(deviceName, (vol) => {
            Marshal.ThrowExceptionForHR(vol.SetMute(muted, IntPtr.Zero));
        });
    }
}
'@

function Get-StateString {
    param([int]$State)
    switch ($State) {
        1 { return "Active" }
        2 { return "Disabled" }
        default { return "Disconnected" }
    }
}

if ($SetVolume -ge 0 -or $GetVolume -or $Mute -or $Unmute -or $GetMute) {
    try {
        if ($SetVolume -ge 0) {
            $level = [Math]::Max(0, [Math]::Min(1, $SetVolume))
            [Audio]::SetVolume($DeviceName, $level)
            Write-Output "Volume set to $([Math]::Round($level * 100))%"
        }
        
        if ($GetVolume) {
            $vol = [Audio]::GetVolume($DeviceName)
            Write-Output $([Math]::Round($vol * 100))
        }
        
        if ($Mute) {
            [Audio]::SetMute($DeviceName, $true)
            Write-Output "Muted"
        }
        
        if ($Unmute) {
            [Audio]::SetMute($DeviceName, $false)
            Write-Output "Unmuted"
        }
        
        if ($GetMute) {
            $muted = [Audio]::GetMute($DeviceName)
            Write-Output $muted
        }
    }
    catch {
        Write-Error "Audio operation failed: $($_.Exception.Message)"
        exit 1
    }
    return
}

# CASE 1: Specific Property Lookup
if ($Id -or $Name) {
    if (-not $Out) {
        Write-Error "You must specify an -Out property (Name, Type, Id, State) when using -Id or -Name."
        exit 1
    }

    $Devices = [Audio]::GetDevices(-1)
    $TargetDevice = $null

    if ($Id) {
        $TargetDevice = $Devices | Where-Object { $_.Id -eq $Id } | Select-Object -First 1
    } 
    elseif ($Name) {
        $TargetDevice = $Devices | Where-Object { $_.Name -eq $Name } | Select-Object -First 1
    }

    if ($TargetDevice) {
        switch ($Out) {
            "Name"  { Write-Output $TargetDevice.Name }
            "Type"  { Write-Output $TargetDevice.Type }
            "Id"    { Write-Output $TargetDevice.Id }
            "State" { Write-Output (Get-StateString -State $TargetDevice.State) }
        }
    } else {
        Write-Error "Device not found matching the provided criteria."
        exit 1
    }
    return
}

# CASE 2: List Devices
if ($ListAll -or $ListOut -or $ListIn) {
    
    $FlowParam = -1
    if ($ListOut) { $FlowParam = 0 }
    elseif ($ListIn) { $FlowParam = 1 }

    $Devices = [Audio]::GetDevices($FlowParam)

    # Apply State Filters
    $AllowedStates = @()
    if ($Active)       { $AllowedStates += 1 }
    if ($Disabled)     { $AllowedStates += 2 }
    if ($Disconnected) { $AllowedStates += 0 }

    if ($AllowedStates.Count -gt 0) {
        $Devices = $Devices | Where-Object { $AllowedStates -contains $_.State }
    }

    # Parse Custom Format string if provided (e.g. "S,N,I")
    # Removes spaces and splits by comma
    $FormatTokens = @()
    if ($F) {
        $FormatTokens = ($F.ToUpper() -replace '\s', '') -split ','
    }

    # Print Headers ONLY if -NF and -F are NOT set
    if (-not $NF -and -not $F) {
        Write-Output ""
        Write-Output ("{0,-12} {1,-45} {2}" -f "Status", "Device Friendly Name", "Device ID")
        Write-Output (New-Object String ('-', 115))
    }

    $LastType = $null
    foreach ($device in $Devices) {
        
        $StateStr = Get-StateString -State $device.State

        # Output logic based on flags
        if ($F) {
            # CUSTOM FORMAT: Build array based on letters provided, then join with comma
            $outParts = @()
            foreach ($t in $FormatTokens) {
                switch ($t) {
                    'S' { $outParts += $StateStr }
                    'N' { $outParts += $device.Name }
                    'I' { $outParts += $device.Id }
                }
            }
            Write-Output ($outParts -join ', ')
        }
        elseif ($NF) {
            # NO FORMATTING: Just output the default columns without headers/separators/categories
            "{0,-12} {1,-45} {2}" -f $StateStr, $device.Name, $device.Id
        }
        else {
            # DEFAULT FORMATTING: Includes Categories and Headers
            if ($device.Type -ne $LastType) {
                Write-Output "`n[$($device.Type)]"
                $LastType = $device.Type
            }
            "{0,-12} {1,-45} {2}" -f $StateStr, $device.Name, $device.Id
        }
    }
    return
}

# CASE 3: No valid parameters passed (Fallback)
Write-Output "No valid parameters passed. Running default behavior (-ListAll)..."
Write-Output ""
Write-Output "Usage examples:"
Write-Output "  Device Listing:"
Write-Output "    ./script.ps1 -ListAll"
Write-Output "    ./script.ps1 -ListOut -Active -NF"
Write-Output '    ./script.ps1 -ListIn -F "N, I"'
Write-Output ""
Write-Output "  Property Lookup:"
Write-Output '  ./script.ps1 -Id "{0.0.1.00000000}.{...}" -Out Name'
Write-Output '  ./script.ps1 -Name "Speakers" -Out State'
Write-Output ""
Write-Output "  Volume/Mute Controls:"
Write-Output "    ./script.ps1 -GetVolume"
Write-Output "    ./script.ps1 -SetVolume 0.5"
Write-Output "    ./script.ps1 -Mute"
Write-Output '    ./script.ps1 -Unmute -DeviceName "Speakers"'
Write-Output "    ./script.ps1 -GetMute"
Write-Output ""
& $MyInvocation.MyCommand.Path -ListAll