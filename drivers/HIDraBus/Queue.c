#include "HIDraBus_VHF.h"

static NTSTATUS HandleCreate(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen, _Out_writes_bytes_(OutLen) PVOID OutBuf, _In_ size_t OutLen)
{
    NTSTATUS status;
    WDFDEVICE device;
    PDEVICE_CONTEXT ctx;
    PHIDRA_CREATE_IN cin;
    PHIDRA_CREATE_OUT cout;
    PHIDRA_VHF_DEVICE vhfDevice;

    if (InLen < sizeof(HIDRA_CREATE_IN) || OutLen < sizeof(HIDRA_CREATE_OUT))
        return STATUS_BUFFER_TOO_SMALL;

    device = WdfIoQueueGetDevice(WdfRequestGetIoQueue(Request));
    ctx = DeviceGetContext(device);
    cin = (PHIDRA_CREATE_IN)InBuf;
    cout = (PHIDRA_CREATE_OUT)OutBuf;

    // Validate device kind
    HIDRA_DEVICE_KIND kind = (HIDRA_DEVICE_KIND)cin->Kind;
    if (kind != HIDRA_KIND_X360 && kind != HIDRA_KIND_DS4 && kind != HIDRA_KIND_DS5) {
        return STATUS_INVALID_PARAMETER;
    }

    // Create VHF device
    status = CreateVhfDevice(device, kind, &vhfDevice);
    if (!NT_SUCCESS(status)) {
        return status;
    }

    // Return the handle
    cout->Handle = vhfDevice->Handle;

    WdfRequestSetInformation(Request, sizeof(HIDRA_CREATE_OUT));
    return STATUS_SUCCESS;
}

static NTSTATUS HandleUpdate(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen)
{
    NTSTATUS status;
    WDFDEVICE device;
    PDEVICE_CONTEXT ctx;
    PHIDRA_UPDATE_IN uin;
    PHIDRA_VHF_DEVICE vhfDevice;

    if (InLen < sizeof(HIDRA_UPDATE_IN))
        return STATUS_BUFFER_TOO_SMALL;

    device = WdfIoQueueGetDevice(WdfRequestGetIoQueue(Request));
    ctx = DeviceGetContext(device);
    uin = (PHIDRA_UPDATE_IN)InBuf;

    // Find the VHF device by handle
    vhfDevice = FindVhfDeviceByHandle(ctx, uin->Handle);
    if (!vhfDevice) {
        return STATUS_INVALID_HANDLE;
    }

    // Update the device state
    status = UpdateVhfDeviceState(vhfDevice, &uin->State);
    if (!NT_SUCCESS(status)) {
        return status;
    }

    WdfRequestSetInformation(Request, 0);
    return STATUS_SUCCESS;
}

static NTSTATUS HandleDestroy(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen)
{
    WDFDEVICE device;
    PDEVICE_CONTEXT ctx;
    PHIDRA_DESTROY_IN din;
    PHIDRA_VHF_DEVICE vhfDevice;

    if (InLen < sizeof(HIDRA_DESTROY_IN))
        return STATUS_BUFFER_TOO_SMALL;

    device = WdfIoQueueGetDevice(WdfRequestGetIoQueue(Request));
    ctx = DeviceGetContext(device);
    din = (PHIDRA_DESTROY_IN)InBuf;

    // Find the VHF device by handle
    vhfDevice = FindVhfDeviceByHandle(ctx, din->Handle);
    if (!vhfDevice) {
        return STATUS_INVALID_HANDLE;
    }

    // Destroy the VHF device
    DestroyVhfDevice(device, vhfDevice);

    WdfRequestSetInformation(Request, 0);
    return STATUS_SUCCESS;
}

VOID EvtIoDeviceControl(
    _In_ WDFQUEUE Queue,
    _In_ WDFREQUEST Request,
    _In_ size_t OutputBufferLength,
    _In_ size_t InputBufferLength,
    _In_ ULONG IoControlCode)
{
    UNREFERENCED_PARAMETER(Queue);
    UNREFERENCED_PARAMETER(OutputBufferLength);
    UNREFERENCED_PARAMETER(InputBufferLength);

    NTSTATUS status = STATUS_INVALID_DEVICE_REQUEST;
    
    PVOID inBuf = NULL, outBuf = NULL;
    size_t inLen = 0, outLen = 0;

    // METHOD_BUFFERED – both via system buffer
    if (IoControlCode == IOCTL_HIDRA_CREATE || IoControlCode == IOCTL_HIDRA_UPDATE || IoControlCode == IOCTL_HIDRA_DESTROY)
    {
        status = WdfRequestRetrieveInputBuffer(Request, 0, &inBuf, &inLen);
        if (!NT_SUCCESS(status))
            goto done;
    }
    if (IoControlCode == IOCTL_HIDRA_CREATE)
    {
        status = WdfRequestRetrieveOutputBuffer(Request, sizeof(HIDRA_CREATE_OUT), &outBuf, &outLen);
        if (!NT_SUCCESS(status))
            goto done;
    }

    switch (IoControlCode)
    {
    case IOCTL_HIDRA_CREATE:
        status = HandleCreate(Request, inBuf, inLen, outBuf, outLen);
        break;
    case IOCTL_HIDRA_UPDATE:
        status = HandleUpdate(Request, inBuf, inLen);
        break;
    case IOCTL_HIDRA_DESTROY:
        status = HandleDestroy(Request, inBuf, inLen);
        break;
    default:
        status = STATUS_INVALID_DEVICE_REQUEST;
        break;
    }

done:
    WdfRequestComplete(Request, status);
}