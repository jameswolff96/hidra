#pragma once
#include <ntddk.h>
#include <wdf.h>
#include <initguid.h> // for DEFINE_GUID
#include <ntstrsafe.h>

//
// --------- Keep these in sync with hidra-protocol (Rust) ---------
// CTL_CODE layout: (DeviceType<<16) | (Access<<14) | (Function<<2) | Method
//
#define HIDRA_DEVICE_TYPE FILE_DEVICE_UNKNOWN
#define HIDRA_IOCTL_BASE 0x800

#define IOCTL_HIDRA_CREATE CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 0, METHOD_BUFFERED, FILE_WRITE_ACCESS)
#define IOCTL_HIDRA_UPDATE CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 1, METHOD_BUFFERED, FILE_WRITE_ACCESS)
#define IOCTL_HIDRA_DESTROY CTL_CODE(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 2, METHOD_BUFFERED, FILE_WRITE_ACCESS)

//
// ABI structs — byte-for-byte with #[repr(C)] in Rust
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
} HIDRA_PAD_STATE;

C_ASSERT(sizeof(HIDRA_PAD_STATE) == 14);

typedef enum _HIDRA_DEVICE_KIND
{
    HIDRA_KIND_X360 = 0,
    HIDRA_KIND_DS4 = 1,
    HIDRA_KIND_DS5 = 2,
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
// Device Interface GUID — MUST match hidra_protocol::HIDRA_INTERFACE_GUID
// Generate once and keep identical on both sides.
//
DEFINE_GUID(GUID_DEVINTERFACE_HIDRA,
    0xA1B2C3D4, 0x1111, 0x2222, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA);
// ^^^ REPLACE with your real GUID from Rust (same 128-bit value)

//
// Device context
//
typedef struct _DEVICE_CONTEXT
{
    ULONGLONG NextHandle;
    // TODO: per-instance state map (WDFCOLLECTION / WDFLOOKASIDE) if needed
} DEVICE_CONTEXT, * PDEVICE_CONTEXT;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(DEVICE_CONTEXT, DeviceGetContext);

//
// Prototypes
//
DRIVER_INITIALIZE DriverEntry;
EVT_WDF_DRIVER_DEVICE_ADD EvtDeviceAdd;
EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL EvtIoDeviceControl;
