#include "HIDraBus.h"
#include <wdmsec.h>

NTSTATUS EvtDeviceAdd(_In_ WDFDRIVER Driver, _Inout_ PWDFDEVICE_INIT DeviceInit)
{
    UNREFERENCED_PARAMETER(Driver);

    NTSTATUS status;
    WDFDEVICE device;
    WDF_OBJECT_ATTRIBUTES attrs;
    PDEVICE_CONTEXT deviceContext;

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attrs, DEVICE_CONTEXT);

    // PnP device (use the DeviceInit provided by the framework)
    status = WdfDeviceCreate(&DeviceInit, &attrs, &device);
    if (!NT_SUCCESS(status)) return status;

    // Get device context and initialize
    deviceContext = DeviceGetContext(device);
    deviceContext->NextHandle = 1;
    InitializeListHead(&deviceContext->DeviceList);
    
    // Create spinlock for device list protection
    WDF_OBJECT_ATTRIBUTES spinlockAttrs;
    WDF_OBJECT_ATTRIBUTES_INIT(&spinlockAttrs);
    status = WdfSpinLockCreate(&spinlockAttrs, &deviceContext->DeviceListLock);
    if (!NT_SUCCESS(status)) return status;

    // Publish interface for user-mode enumeration
    status = WdfDeviceCreateDeviceInterface(device, &GUID_DEVINTERFACE_HIDRA, NULL);
    if (!NT_SUCCESS(status)) return status;

    // Default IOCTL queue
    WDF_IO_QUEUE_CONFIG qcfg;
    WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&qcfg, WdfIoQueueDispatchParallel);
    qcfg.EvtIoDeviceControl = EvtIoDeviceControl;

    status = WdfIoQueueCreate(device, &qcfg, WDF_NO_OBJECT_ATTRIBUTES, WDF_NO_HANDLE);
    if (!NT_SUCCESS(status)) return status;

    return STATUS_SUCCESS;
}