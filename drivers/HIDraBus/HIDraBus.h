#pragma once
#include <ntddk.h>
#include <wdf.h>
#include <ntstrsafe.h>
#include <vhf.h>

//
// --------- Keep these in sync with hidra-protocol (Rust) ---------
// CTL_CODE layout: (DeviceType<<16) | (Access<<14) | (Function<<2) | Method
//
#define HIDRA_DEVICE_TYPE FILE_DEVICE_UNKNOWN
#define HIDRA_IOCTL_BASE 0x800

#define IOCTL_HIDRA_CREATE CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 0, METHOD_BUFFERED, FILE_WRITE_ACCESS)
#define IOCTL_HIDRA_UPDATE CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 1, METHOD_BUFFERED, FILE_WRITE_ACCESS)
#define IOCTL_HIDRA_DESTROY CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 2, METHOD_BUFFERED, FILE_WRITE_ACCESS)

EXTERN_C const GUID GUID_DEVINTERFACE_HIDRA;

//
// ABI structs – byte-for-byte with #[repr(C)] in Rust
//
typedef struct _HIDRA_PAD_STATE
{
    UINT16 Buttons;  // 0..1
    SHORT Lx;        // 2..3
    SHORT Ly;        // 4..5
    SHORT Rx;        // 6..7
    SHORT Ry;        // 8..9
    UINT16 Lt;       // 10..11
    UINT16 Rt;       // 12..13
} HIDRA_PAD_STATE, *PHIDRA_PAD_STATE;

C_ASSERT(sizeof(HIDRA_PAD_STATE) == 14);

typedef enum _HIDRA_DEVICE_KIND
{
    HIDRA_KIND_X360 = 0x0366,
    HIDRA_KIND_DS4 = 0x05C4,
    HIDRA_KIND_DS5 = 0x0CE6,
} HIDRA_DEVICE_KIND;

typedef struct _HIDRA_CREATE_IN
{
    ULONG Kind;     // HIDRA_DEVICE_KIND
    ULONG Features; // bitflags
} HIDRA_CREATE_IN, * PHIDRA_CREATE_IN;

typedef struct _HIDRA_CREATE_OUT
{
    ULONGLONG Handle; // driver-assigned
} HIDRA_CREATE_OUT, * PHIDRA_CREATE_OUT;

typedef struct _HIDRA_UPDATE_IN
{
    ULONGLONG Handle;
    HIDRA_PAD_STATE State;
} HIDRA_UPDATE_IN, * PHIDRA_UPDATE_IN;

typedef struct _HIDRA_DESTROY_IN
{
    ULONGLONG Handle;
} HIDRA_DESTROY_IN, * PHIDRA_DESTROY_IN;

//
// VHF Device Instance
//
typedef struct _HIDRA_VHF_DEVICE
{
    ULONGLONG Handle;
    HIDRA_DEVICE_KIND Kind;
    VHFHANDLE VhfHandle;
    UCHAR ReportBuffer[64];  // Max report size
    LIST_ENTRY ListEntry;
} HIDRA_VHF_DEVICE, *PHIDRA_VHF_DEVICE;

//
// Device context
//
typedef struct _DEVICE_CONTEXT
{
    ULONGLONG NextHandle;
    LIST_ENTRY DeviceList;      // List of HIDRA_VHF_DEVICE
    WDFSPINLOCK DeviceListLock; // Protect device list
} DEVICE_CONTEXT, * PDEVICE_CONTEXT;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(DEVICE_CONTEXT, DeviceGetContext);

//
// VHF Report Descriptors
//
extern const UCHAR X360_HID_REPORT_DESCRIPTOR[];
extern const ULONG X360_HID_REPORT_DESCRIPTOR_SIZE;
extern const UCHAR DS4_HID_REPORT_DESCRIPTOR[];
extern const ULONG DS4_HID_REPORT_DESCRIPTOR_SIZE;

//
// Prototypes
//
DRIVER_INITIALIZE DriverEntry;
EVT_WDF_DRIVER_DEVICE_ADD EvtDeviceAdd;
EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL EvtIoDeviceControl;

// VHF functions
EVT_VHF_READY_FOR_NEXT_READ_REPORT EvtVhfReadyForNextReadReport;

NTSTATUS CreateVhfDevice(_In_ WDFDEVICE Device, _In_ HIDRA_DEVICE_KIND Kind, _Out_ PHIDRA_VHF_DEVICE* VhfDevice);
VOID DestroyVhfDevice(_In_ WDFDEVICE Device, _In_ PHIDRA_VHF_DEVICE VhfDevice);
PHIDRA_VHF_DEVICE FindVhfDeviceByHandle(_In_ PDEVICE_CONTEXT Context, _In_ ULONGLONG Handle);
NTSTATUS UpdateVhfDeviceState(_In_ PHIDRA_VHF_DEVICE VhfDevice, _In_ PHIDRA_PAD_STATE State);