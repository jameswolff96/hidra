#include "HIDraBus_VHF.h"

//
// VHF Event Handlers
//

VOID EvtVhfReadyForNextReadReport(
    _In_ VHFHANDLE VhfHandle,
    _In_opt_ PVOID VhfContext)
{
    UNREFERENCED_PARAMETER(VhfHandle);
    UNREFERENCED_PARAMETER(VhfContext);
    // This is called when VHF is ready for the next input report
    // We don't need to do anything special here since we push reports on-demand
}

NTSTATUS EvtVhfAsyncOperation(
    _In_ VHFHANDLE VhfHandle,
    _In_ VHFOPERATIONHANDLE VhfOperationHandle,
    _In_opt_ PVOID VhfContext,
    _In_ PHID_XFER_PACKET HidTransferPacket)
{
    UNREFERENCED_PARAMETER(VhfHandle);
    UNREFERENCED_PARAMETER(VhfOperationHandle);
    UNREFERENCED_PARAMETER(VhfContext);
    UNREFERENCED_PARAMETER(HidTransferPacket);
    
    // Handle async operations like Get/Set Feature reports
    // For basic gamepad functionality, we just return success
    return STATUS_SUCCESS;
}

//
// VHF Device Management
//

NTSTATUS CreateVhfDevice(
    _In_ WDFDEVICE Device,
    _In_ HIDRA_DEVICE_KIND Kind,
    _Out_ PHIDRA_VHF_DEVICE* VhfDevice)
{
    NTSTATUS status;
    PHIDRA_VHF_DEVICE device;
    VHF_CONFIG vhfConfig;
    PDEVICE_CONTEXT deviceContext = DeviceGetContext(Device);
    
    // Allocate our VHF device structure
    device = (PHIDRA_VHF_DEVICE)ExAllocatePoolZero(
        NonPagedPool,
        sizeof(HIDRA_VHF_DEVICE),
        'VhfD');
    
    if (!device) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Initialize VHF configuration
    VHF_CONFIG_INIT(&vhfConfig,
                    WdfDeviceWdmGetDeviceObject(Device),
                    Kind == HIDRA_KIND_X360 ? X360_HID_REPORT_DESCRIPTOR_SIZE : DS4_HID_REPORT_DESCRIPTOR_SIZE,
                    Kind == HIDRA_KIND_X360 ? X360_HID_REPORT_DESCRIPTOR : DS4_HID_REPORT_DESCRIPTOR);

    // Set up VHF callbacks
    vhfConfig.VhfClientContext = device;
    vhfConfig.EvtVhfReadyForNextReadReport = EvtVhfReadyForNextReadReport;
    vhfConfig.EvtVhfAsyncOperation = EvtVhfAsyncOperation;

    // Set device attributes based on controller type
    switch (Kind) {
    case HIDRA_KIND_X360:
        vhfConfig.VendorID = 0x045E;    // Microsoft
        vhfConfig.ProductID = 0x028E;   // Xbox 360 Controller
        vhfConfig.VersionNumber = 0x0114;
        break;
    case HIDRA_KIND_DS4:
        vhfConfig.VendorID = 0x054C;    // Sony
        vhfConfig.ProductID = 0x05C4;   // DualShock 4
        vhfConfig.VersionNumber = 0x0100;
        break;
    case HIDRA_KIND_DS5:
        vhfConfig.VendorID = 0x054C;    // Sony
        vhfConfig.ProductID = 0x0CE6;   // DualSense
        vhfConfig.VersionNumber = 0x0100;
        break;
    default:
        ExFreePoolWithTag(device, 'VhfD');
        return STATUS_INVALID_PARAMETER;
    }

    // Create the VHF device
    status = VhfCreate(&vhfConfig, &device->VhfHandle);
    if (!NT_SUCCESS(status)) {
        ExFreePoolWithTag(device, 'VhfD');
        return status;
    }

    // Start the VHF device
    status = VhfStart(device->VhfHandle);
    if (!NT_SUCCESS(status)) {
        VhfDelete(device->VhfHandle, TRUE);
        ExFreePoolWithTag(device, 'VhfD');
        return status;
    }

    // Initialize our device structure
    device->Handle = deviceContext->NextHandle++;
    device->Kind = Kind;
    RtlZeroMemory(device->ReportBuffer, sizeof(device->ReportBuffer));

    // Add to device list
    WdfSpinLockAcquire(deviceContext->DeviceListLock);
    InsertTailList(&deviceContext->DeviceList, &device->ListEntry);
    WdfSpinLockRelease(deviceContext->DeviceListLock);

    *VhfDevice = device;
    return STATUS_SUCCESS;
}

VOID DestroyVhfDevice(
    _In_ WDFDEVICE Device,
    _In_ PHIDRA_VHF_DEVICE VhfDevice)
{
    PDEVICE_CONTEXT deviceContext = DeviceGetContext(Device);

    // Remove from device list
    WdfSpinLockAcquire(deviceContext->DeviceListLock);
    RemoveEntryList(&VhfDevice->ListEntry);
    WdfSpinLockRelease(deviceContext->DeviceListLock);

    // Stop and delete VHF device
    if (VhfDevice->VhfHandle) {
        VhfDelete(VhfDevice->VhfHandle, TRUE);
    }

    // Free our structure
    ExFreePoolWithTag(VhfDevice, 'VhfD');
}

PHIDRA_VHF_DEVICE FindVhfDeviceByHandle(
    _In_ PDEVICE_CONTEXT Context,
    _In_ ULONGLONG Handle)
{
    PLIST_ENTRY entry;
    PHIDRA_VHF_DEVICE device;

    WdfSpinLockAcquire(Context->DeviceListLock);

    for (entry = Context->DeviceList.Flink;
         entry != &Context->DeviceList;
         entry = entry->Flink) {
        
        device = CONTAINING_RECORD(entry, HIDRA_VHF_DEVICE, ListEntry);
        if (device->Handle == Handle) {
            WdfSpinLockRelease(Context->DeviceListLock);
            return device;
        }
    }

    WdfSpinLockRelease(Context->DeviceListLock);
    return NULL;
}

NTSTATUS UpdateVhfDeviceState(
    _In_ PHIDRA_VHF_DEVICE VhfDevice,
    _In_ PHIDRA_PAD_STATE State)
{
    NTSTATUS status;
    HID_XFER_PACKET hidPacket;
    
    // Pack the state into HID report format based on device type
    switch (VhfDevice->Kind) {
    case HIDRA_KIND_X360:
        {
            // Xbox 360 controller report format
            // Report ID (1 byte) + Report Data (varies)
            VhfDevice->ReportBuffer[0] = 0x01; // Report ID
            
            // Buttons (2 bytes, little endian)
            VhfDevice->ReportBuffer[1] = (UCHAR)(State->Buttons & 0xFF);
            VhfDevice->ReportBuffer[2] = (UCHAR)((State->Buttons >> 8) & 0xFF);
            
            // Triggers (2 bytes)
            VhfDevice->ReportBuffer[3] = (UCHAR)(State->Lt >> 8); // Scale to 0-255
            VhfDevice->ReportBuffer[4] = (UCHAR)(State->Rt >> 8); // Scale to 0-255
            
            // Left stick (4 bytes, little endian)
            VhfDevice->ReportBuffer[5] = (UCHAR)(State->Lx & 0xFF);
            VhfDevice->ReportBuffer[6] = (UCHAR)((State->Lx >> 8) & 0xFF);
            VhfDevice->ReportBuffer[7] = (UCHAR)(State->Ly & 0xFF);
            VhfDevice->ReportBuffer[8] = (UCHAR)((State->Ly >> 8) & 0xFF);
            
            // Right stick (4 bytes, little endian)
            VhfDevice->ReportBuffer[9] = (UCHAR)(State->Rx & 0xFF);
            VhfDevice->ReportBuffer[10] = (UCHAR)((State->Rx >> 8) & 0xFF);
            VhfDevice->ReportBuffer[11] = (UCHAR)(State->Ry & 0xFF);
            VhfDevice->ReportBuffer[12] = (UCHAR)((State->Ry >> 8) & 0xFF);
            
            hidPacket.reportBufferLen = 13; // Report ID + 12 bytes of data
        }
        break;
        
    case HIDRA_KIND_DS4:
    case HIDRA_KIND_DS5:
        {
            // DS4/DS5 controller report format (simplified)
            VhfDevice->ReportBuffer[0] = 0x01; // Report ID
            
            // Left stick (scaled to 0-255)
            VhfDevice->ReportBuffer[1] = (UCHAR)((State->Lx + 32768) >> 8);
            VhfDevice->ReportBuffer[2] = (UCHAR)((State->Ly + 32768) >> 8);
            
            // Right stick (scaled to 0-255)  
            VhfDevice->ReportBuffer[3] = (UCHAR)((State->Rx + 32768) >> 8);
            VhfDevice->ReportBuffer[4] = (UCHAR)((State->Ry + 32768) >> 8);
            
            // Triggers
            VhfDevice->ReportBuffer[5] = (UCHAR)(State->Lt >> 8);
            VhfDevice->ReportBuffer[6] = (UCHAR)(State->Rt >> 8);
            
            // Buttons (2 bytes)
            VhfDevice->ReportBuffer[7] = (UCHAR)(State->Buttons & 0xFF);
            VhfDevice->ReportBuffer[8] = (UCHAR)((State->Buttons >> 8) & 0xFF);
            
            hidPacket.reportBufferLen = 9; // Simplified report
        }
        break;
        
    default:
        return STATUS_INVALID_PARAMETER;
    }

    // Set up HID transfer packet
    hidPacket.reportBuffer = VhfDevice->ReportBuffer;
    hidPacket.reportId = VhfDevice->ReportBuffer[0];

    // Submit the report to VHF
    status = VhfReadReportSubmit(VhfDevice->VhfHandle, &hidPacket);
    
    return status;
}