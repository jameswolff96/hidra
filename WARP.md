# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

HIDra is a virtual controller system for Windows that creates virtual game controllers (X360, DS4, DS5) that can be controlled programmatically. The system consists of user-mode Rust crates that communicate with Windows kernel-mode drivers to present virtual HID devices to games.

## Common Commands

### Building
```powershell
# Build all crates (debug)
cargo build --workspace

# Build release version
cargo build --workspace --release

# Or use the helper script
.\scripts\Build-All.ps1           # debug build
.\scripts\Build-All.ps1 -Release  # release build
```

### Testing
```powershell
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p hidra-protocol
cargo test -p hidra-broker
cargo test -p hidra-client

# Run a single test
cargo test --workspace test_name
```

### Linting and Code Quality
```powershell
# Run clippy on all targets
cargo clippy --workspace --all-targets -- -D warnings

# Format code
cargo fmt --all
```

### Running Components
```powershell
# Start the broker (server daemon)
cargo run -p hidra-broker

# Use the CLI tools
cargo run -p hidra-tools -- spawn x360     # Create X360 controller
cargo run -p hidra-tools -- ping           # Test broker connection
cargo run -p hidra-tools -- destroy 123    # Destroy controller with handle 123

# Enable backend-driver feature for actual driver communication
cargo run -p hidra-broker --features backend-driver
```

### Driver Development (Windows-specific)
```powershell
# Enable Windows test signing (required for unsigned drivers)
.\scripts\Enable-TestSigning.ps1

# Create development code-signing certificate
.\scripts\New-DevCert.ps1
```

## Architecture

The system follows a layered architecture:

### Core Layers
- **hidra-protocol**: ABI-stable protocol definitions, IOCTLs, and device types (X360, DS4, DS5)
- **hidra-ipc**: Named pipe communication protocol between client and broker
- **hidra-broker**: Server daemon that manages virtual devices, supports mock and driver backends
- **hidra-client**: High-level async client library for creating and managing virtual controllers
- **hidra-ffi**: C-compatible FFI bindings for integration with other languages
- **hidra-tools**: CLI utilities for testing and debugging

### Backend System
The broker supports two backends:
- **Mock backend** (default): For testing without drivers, logs controller state changes
- **Driver backend** (`backend-driver` feature): Communicates with Windows KMDF drivers via IOCTLs

### Communication Flow
1. Client applications use `hidra-client` or `hidra-ffi`
2. Requests are serialized via `hidra-ipc` over Windows named pipes
3. `hidra-broker` processes requests using the configured backend
4. Mock backend logs state; driver backend creates actual virtual HID devices
5. Games see virtual controllers as real hardware

### Device Management
- Each virtual device has a unique handle for identification
- Supports concurrent management of multiple virtual controllers
- State updates are pumped at 250Hz (4ms intervals) to maintain responsiveness
- Device lifecycle: Create → Update states → Destroy

### Driver Integration
- Uses Windows Device Installation API to enumerate HIDra driver interfaces
- Communicates via custom IOCTLs (CREATE, UPDATE, DESTROY)
- Supports multiple simultaneous virtual devices
- Requires proper driver signing for production use

## Development Notes

### Feature Flags
- Use `--features backend-driver` when building/running broker to enable real driver communication
- Default mock backend is suitable for development and testing without drivers

### Testing Virtual Controllers
Use `hidra-tools quick-probe` for a complete lifecycle test (create DS4 → update state → destroy).

### Debugging
- Set `RUST_LOG=debug` environment variable for detailed tracing
- Mock backend logs all device operations for debugging
- Driver backend includes debug traces for IOCTL operations

### Platform Requirements
- Windows-only (uses Windows-specific APIs: named pipes, Windows drivers, IOCTL)
- Requires elevated permissions for driver operations
- Development requires Windows SDK and appropriate driver signing setup