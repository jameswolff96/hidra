#include "HIDraBus.h"
#include <wdmsec.h>

static NTSTATUS CreateDevice(_In_ WDFDRIVER Driver, _Out_ WDFDEVICE* Device) {
    UNREFERENCED_PARAMETER(Driver);

    WDFDEVICE device;
    WDF_OBJECT_ATTRIBUTES attrs;
    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attrs, DEVICE_CONTEXT);

    PWDFDEVICE_INIT pInit = WdfControlDeviceInitAllocate(Driver, &SDDL_DEVOBJ_SYS_ALL_ADM_ALL);
    if (!pInit) return STATUS_INSUFFICIENT_RESOURCES;

    // Device name + symbolic link are optional when you publish a device interface
    NTSTATUS status = WdfDeviceCreate(&pInit, &attrs, &device);
    if (!NT_SUCCESS(status)) {
        WdfDeviceInitFree(pInit);
        return status;
    }

    // Publish interface GUID so user-mode can find/open it
    status = WdfDeviceCreateDeviceInterface(device, &GUID_DEVINTERFACE_HIDRA, NULL);
    if (!NT_SUCCESS(status)) {
        WdfObjectDelete(device);
        return status;
    }

    // Default I/O queue (parallel) for IOCTLs
    WDF_IO_QUEUE_CONFIG qcfg;
    WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&qcfg, WdfIoQueueDispatchParallel);
    qcfg.EvtIoDeviceControl = EvtIoDeviceControl;

    status = WdfIoQueueCreate(device, &qcfg, WDF_NO_OBJECT_ATTRIBUTES, WDF_NO_HANDLE);
    if (!NT_SUCCESS(status)) {
        WdfObjectDelete(device);
        return status;
    }

    // Init context
    PDEVICE_CONTEXT ctx = DeviceGetContext(device);
    ctx->NextHandle = 1;

    // Make device visible
    WdfControlFinishInitializing(device);
    *Device = device;
    return STATUS_SUCCESS;
}

NTSTATUS EvtDeviceAdd(_In_ WDFDRIVER Driver, _Inout_ PWDFDEVICE_INIT DeviceInit)
{
    UNREFERENCED_PARAMETER(Driver);

    NTSTATUS status;
    WDFDEVICE device;
    WDF_OBJECT_ATTRIBUTES attrs;
    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attrs, DEVICE_CONTEXT);

    // PnP device (use the DeviceInit provided by the framework)
    status = WdfDeviceCreate(&DeviceInit, &attrs, &device);
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

    DeviceGetContext(device)->NextHandle = 1;
    return STATUS_SUCCESS;
}

