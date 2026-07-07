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
    [switch]$NF,
    [string]$F,
    [string]$Id,
    [string]$Name,
    
    [string]$SetVolume,
    [switch]$GetVolume,
    [switch]$Mute,
    [switch]$Unmute,
    [switch]$GetMute,
    [string]$DeviceName, # Kept for backwards compatibility
    
    [ValidateSet("Name", "Type", "Id", "State")]
    [string]$Out
)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;
using System.Globalization;

[Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IAudioEndpointVol
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

[Guid("886d8eeb-8cf2-4446-8d02-cdba1dbdcf99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IPropStore
{
    [PreserveSig] int GetCount(out uint cProps);
    [PreserveSig] int GetAt(uint iProp, out PropKey pkey);
    [PreserveSig] int GetValue(ref PropKey key, out PropVar pv);
    [PreserveSig] int SetValue(ref PropKey key, ref PropVar pv);
    [PreserveSig] int Commit();
}

[StructLayout(LayoutKind.Sequential)]
internal struct PropKey
{
    public Guid fmtid;
    public UInt32 pid;
}

[StructLayout(LayoutKind.Explicit)]
internal struct PropVar
{
    [FieldOffset(0)] public ushort vt;
    [FieldOffset(8)] public IntPtr pointerVal;
}

[ComImport]
[Guid("D666063F-1587-4E43-81F1-B948E807363F")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IMMDev
{
    [PreserveSig]
    int Activate(ref Guid iid, uint dwClsCtx, IntPtr pActivationParams, [MarshalAs(UnmanagedType.Interface)] out object ppInterface);
    
    [PreserveSig] 
    int OpenPropertyStore(uint stgmAccess, out IPropStore properties);

    [PreserveSig]
    int GetId([MarshalAs(UnmanagedType.LPWStr)] out string ppstrId);

    [PreserveSig]
    int GetState(out int pdwState);
}

[Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevCollection
{
    int GetCount(out uint count);
    int Item(uint index, out IMMDev device);
}

[Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevEnumerator
{
    int EnumAudioEndpoints(int dataFlow, int stateMask, out IMMDevCollection collection);
    int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDev endpoint);
}

[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")] 
class MMDevEnumComObj { }

public class AudioDevice
{
    public string Id { get; set; }
    public string Name { get; set; }
    public string Type { get; set; }
    public int State { get; set; }
}

public class AudioCtrl
{
    private static PropKey PKEY_Device_FriendlyName = new PropKey
    {
        fmtid = new Guid("a45c254e-df1c-4efd-8020-67d146a850e0"),
        pid = 14
    };

    private static string GetDeviceName(IMMDev device)
    {
        IPropStore store;
        if (device.OpenPropertyStore(0, out store) == 0)
        {
            PropVar prop;
            if (store.GetValue(ref PKEY_Device_FriendlyName, out prop) == 0)
            {
                if (prop.vt == 31)
                {
                    string name = Marshal.PtrToStringUni(prop.pointerVal);
                    Marshal.FreeCoTaskMem(prop.pointerVal);
                    return name;
                }
            }
            Marshal.ReleaseComObject(store);
        }
        return "Unknown Device";
    }

    public static List<AudioDevice> GetDevices(int dataFlow)
    {
        var devices = new List<AudioDevice>();
        IMMDevEnumerator enumerator = (IMMDevEnumerator)new MMDevEnumComObj();
        
        int[] flowsToEnum = (dataFlow == -1) ? new int[] { 0, 1 } : new int[] { dataFlow };
        string[] flowNames = { "Playback (Output)", "Recording (Input)" };
        
        try
        {
            for (int f = 0; f < flowsToEnum.Length; f++)
            {
                int flow = flowsToEnum[f];
                IMMDevCollection collection;
                Marshal.ThrowExceptionForHR(enumerator.EnumAudioEndpoints(flow, 7, out collection));
                try
                {
                    uint count;
                    Marshal.ThrowExceptionForHR(collection.GetCount(out count));
                    
                    for (uint i = 0; i < count; i++)
                    {
                        IMMDev device;
                        Marshal.ThrowExceptionForHR(collection.Item(i, out device));
                        try
                        {
                            string deviceId;
                            Marshal.ThrowExceptionForHR(device.GetId(out deviceId));
                            
                            string name = GetDeviceName(device);

                            int state;
                            Marshal.ThrowExceptionForHR(device.GetState(out state));
                            
                            devices.Add(new AudioDevice
                            {
                                Id = deviceId,
                                Name = name,
                                Type = flowNames[f],
                                State = state
                            });
                        }
                        finally { Marshal.ReleaseComObject(device); }
                    }
                }
                finally { Marshal.ReleaseComObject(collection); }
            }
        }
        finally { Marshal.ReleaseComObject(enumerator); }
        
        return devices;
    }

    public static void ExecuteVolumeAction(string deviceIdentifier, Action<IAudioEndpointVol> action)
    {
        IMMDevEnumerator enumerator = (IMMDevEnumerator)new MMDevEnumComObj();
        IMMDev device = null;

        try
        {
            if (string.IsNullOrEmpty(deviceIdentifier))
            {
                Marshal.ThrowExceptionForHR(enumerator.GetDefaultAudioEndpoint(0, 0, out device));
            }
            else
            {
                IMMDevCollection collection;
                Marshal.ThrowExceptionForHR(enumerator.EnumAudioEndpoints(0, 7, out collection));
                try
                {
                    uint count;
                    Marshal.ThrowExceptionForHR(collection.GetCount(out count));
                    bool found = false;
                    for (uint i = 0; i < count; i++)
                    {
                        IMMDev d;
                        Marshal.ThrowExceptionForHR(collection.Item(i, out d));
                        try
                        {
                            string name = GetDeviceName(d);
                            string id = null;
                            Marshal.ThrowExceptionForHR(d.GetId(out id));

                            // UPDATED: Now matches against BOTH Name and ID
                            if (string.Equals(name, deviceIdentifier, StringComparison.OrdinalIgnoreCase) ||
                                string.Equals(id, deviceIdentifier, StringComparison.OrdinalIgnoreCase))
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
                    if (!found) throw new Exception("Device not found: " + deviceIdentifier);
                }
                finally { Marshal.ReleaseComObject(collection); }
            }

            object volumeObj = null;
            Guid epvid = typeof(IAudioEndpointVol).GUID;
            Marshal.ThrowExceptionForHR(device.Activate(ref epvid, 23, IntPtr.Zero, out volumeObj));
            
            if (volumeObj != null)
            {
                IAudioEndpointVol endpointVolume = (IAudioEndpointVol)volumeObj;
                action(endpointVolume);
                Marshal.ReleaseComObject(endpointVolume);
            }
        }
        finally
        {
            if (device != null) Marshal.ReleaseComObject(device);
            Marshal.ReleaseComObject(enumerator);
        }
    }

    public static float GetVolume(string deviceIdentifier = null)
    {
        float v = -1f;
        ExecuteVolumeAction(deviceIdentifier, (vol) => {
            Marshal.ThrowExceptionForHR(vol.GetMasterVolumeLevelScalar(out v));
        });
        return v;
    }

    public static void ApplyVolume(string deviceIdentifier, string rawInput)
    {
        string cleanInput = rawInput.Trim().Replace(',', '.');
        double level = double.Parse(cleanInput, CultureInfo.InvariantCulture);
        
        if (level > 1.0) level = level / 100.0;
        
        float fLevel = (float)Math.Max(0.0, Math.Min(1.0, level));
        
        ExecuteVolumeAction(deviceIdentifier, (vol) => {
            Marshal.ThrowExceptionForHR(vol.SetMasterVolumeLevelScalar(fLevel, IntPtr.Zero));
        });
    }

    public static bool GetMute(string deviceIdentifier = null)
    {
        bool mute = false;
        ExecuteVolumeAction(deviceIdentifier, (vol) => {
            Marshal.ThrowExceptionForHR(vol.GetMute(out mute));
        });
        return mute;
    }

    public static void SetMute(string deviceIdentifier, bool muted)
    {
        ExecuteVolumeAction(deviceIdentifier, (vol) => {
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

if (($PSBoundParameters.ContainsKey('SetVolume') -and ![string]::IsNullOrWhiteSpace($SetVolume)) -or $GetVolume -or $Mute -or $Unmute -or $GetMute) {
    try {
        # Priority: -Id first, then -Name, then legacy -DeviceName
        $TargetDevice = if ($Id) { $Id } elseif ($Name) { $Name } else { $DeviceName }

        if ($PSBoundParameters.ContainsKey('SetVolume') -and ![string]::IsNullOrWhiteSpace($SetVolume)) {
            [AudioCtrl]::ApplyVolume($TargetDevice, $SetVolume)
            
            $vol = [AudioCtrl]::GetVolume($TargetDevice)
            Write-Output "Volume set to $([Math]::Round($vol * 100))%"
        }
        
        if ($GetVolume) {
            $vol = [AudioCtrl]::GetVolume($TargetDevice)
            Write-Output $([Math]::Round($vol * 100))
        }
        
        if ($Mute) {
            [AudioCtrl]::SetMute($TargetDevice, $true)
            Write-Output "Muted"
        }
        
        if ($Unmute) {
            [AudioCtrl]::SetMute($TargetDevice, $false)
            Write-Output "Unmuted"
        }
        
        if ($GetMute) {
            $muted = [AudioCtrl]::GetMute($TargetDevice)
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

    $Devices = [AudioCtrl]::GetDevices(-1)
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

    $Devices = [AudioCtrl]::GetDevices($FlowParam)

    $AllowedStates = @()
    if ($Active)       { $AllowedStates += 1 }
    if ($Disabled)     { $AllowedStates += 2 }
    if ($Disconnected) { $AllowedStates += 0 }

    if ($AllowedStates.Count -gt 0) {
        $Devices = $Devices | Where-Object { $AllowedStates -contains $_.State }
    }

    $FormatTokens = @()
    if ($F) {
        $FormatTokens = ($F.ToUpper() -replace '\s', '') -split ','
    }

    if (-not $NF -and -not $F) {
        Write-Output ""
        Write-Output ("{0,-12} {1,-45} {2}" -f "Status", "Device Friendly Name", "Device ID")
        Write-Output (New-Object String ('-', 115))
    }

    $LastType = $null
    foreach ($device in $Devices) {
        
        $StateStr = Get-StateString -State $device.State

        if ($F) {
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
            "{0,-12} {1,-45} {2}" -f $StateStr, $device.Name, $device.Id
        }
        else {
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
Write-Output '    ./script.ps1 -Id "{0.0.1.00000000}.{...}" -Out Name'
Write-Output '    ./script.ps1 -Name "Speakers" -Out State'
Write-Output ""
Write-Output "  Volume/Mute Controls (Accepts 0-100 for percentage):"
Write-Output "    ./script.ps1 -GetVolume"
Write-Output "    ./script.ps1 -SetVolume 50"
Write-Output '    ./script.ps1 -SetVolume 80 -Name "Speakers"'
Write-Output '    ./script.ps1 -Mute -Id "{0.0.1.00000000}.{...}"'
Write-Output "    ./script.ps1 -GetMute"
Write-Output ""
& $MyInvocation.MyCommand.Path -ListAll