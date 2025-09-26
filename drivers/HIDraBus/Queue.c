#include "HIDraBus.h"

static NTSTATUS HandleCreate(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen, _Out_writes_bytes_(OutLen) PVOID OutBuf, _In_ size_t OutLen)
{
    if (InLen < sizeof(HIDRA_CREATE_IN) || OutLen < sizeof(HIDRA_CREATE_OUT))
        return STATUS_BUFFER_TOO_SMALL;

    WDFDEVICE device = WdfIoQueueGetDevice(WdfRequestGetIoQueue(Request));
    PDEVICE_CONTEXT ctx = DeviceGetContext(device);

    PHIDRA_CREATE_IN cin = (PHIDRA_CREATE_IN)InBuf;
    PHIDRA_CREATE_OUT cout = (PHIDRA_CREATE_OUT)OutBuf;

    UNREFERENCED_PARAMETER(cin); // kind/features available here for future VHF init

    ULONGLONG handle = ctx->NextHandle++;
    cout->Handle = handle;

    WdfRequestSetInformation(Request, sizeof(HIDRA_CREATE_OUT));
    return STATUS_SUCCESS;
}

static NTSTATUS HandleUpdate(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen)
{
    if (InLen < sizeof(HIDRA_UPDATE_IN))
        return STATUS_BUFFER_TOO_SMALL;

    PHIDRA_UPDATE_IN uin = (PHIDRA_UPDATE_IN)InBuf;

    // TODO: look up instance by uin->Handle and submit to VHF (VhfWriteReportData)
    UNREFERENCED_PARAMETER(uin);

    WdfRequestSetInformation(Request, 0);
    return STATUS_SUCCESS;
}

static NTSTATUS HandleDestroy(_In_ WDFREQUEST Request, _In_reads_bytes_(InLen) PVOID InBuf, _In_ size_t InLen)
{
    if (InLen < sizeof(HIDRA_DESTROY_IN))
        return STATUS_BUFFER_TOO_SMALL;

    PHIDRA_DESTROY_IN din = (PHIDRA_DESTROY_IN)InBuf;
    UNREFERENCED_PARAMETER(din);
    // TODO: free per-instance resources / stop VHF target

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

    // METHOD_BUFFERED — both via system buffer
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
