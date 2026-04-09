# SAE J1979 OBD-II Service Modes

Diagnostic service definitions per SAE J1979 (ISO 15031-5). These are the standard OBD-II request modes supported by all compliant vehicles.

## Service Mode Summary

| Mode | Hex | Name | Description | Response |
|------|-----|------|-------------|----------|
| 01 | `01` | Current Data | Request current powertrain diagnostic data | `41` + PID + data |
| 02 | `02` | Freeze Frame | Request freeze frame data (snapshot at DTC time) | `42` + PID + frame + data |
| 03 | `03` | Stored DTCs | Request emission-related stored DTCs | `43` + DTC pairs |
| 04 | `04` | Clear DTCs | Clear/reset emission-related diagnostic information | `44` |
| 05 | `05` | O2 Sensor Test | Request oxygen sensor monitoring test results | `45` + data |
| 06 | `06` | On-Board Test | Request on-board monitoring test results (non-continuous) | `46` + data |
| 07 | `07` | Pending DTCs | Request emission-related DTCs detected during current drive | `47` + DTC pairs |
| 08 | `08` | Control Operation | Request control of on-board system/test/component | `48` + data |
| 09 | `09` | Vehicle Info | Request vehicle information (VIN, calibration IDs, etc.) | `49` + InfoType + data |
| 0A | `0A` | Permanent DTCs | Request permanent emission-related DTCs (cannot be cleared by scan tool) | `4A` + DTC pairs |

## Response Mode Calculation

```
Response mode byte = Request mode + 0x40

Example: Request mode 01 -> Response starts with 41
```

## Service $01 - Current Data

### Request Format
```
01 [PID]
```

### PID Support Discovery
Every 32 PIDs, a "supported PIDs" bitmap is available:

| PID | Reports Support For |
|-----|-------------------|
| `00` | PIDs 01-20 |
| `20` | PIDs 21-40 |
| `40` | PIDs 41-60 |
| `60` | PIDs 61-80 |
| `80` | PIDs 81-A0 |
| `A0` | PIDs A1-C0 |
| `C0` | PIDs C1-E0 |

Request: `01 00`
Response: `41 00 BE 1F B8 10` (4 bytes = 32 bits, each bit = supported PID)

### Optimization: Response Count
Append a single hex digit to specify expected response count:
```
01 0C 1      -- RPM, expect 1 ECU response (faster)
01 00 1      -- Supported PIDs, expect 1 response
```

### Multi-ECU Responses
Multiple ECUs may respond to the same PID. Enable headers (`AT H1`) and check the source address byte to determine which ECU sent each response.

## Service $02 - Freeze Frame Data

### Request Format
```
02 [PID] [Frame Number]
```

Frame number is typically `00`. Same PIDs as Service $01 but returns data captured at the time a DTC was set.

### Example
```
02 0C 00     -- Get RPM from freeze frame 0
Response: 42 0C 00 1A F8
```

## Service $03 - Stored DTCs

### Request Format
```
03
```

No PID needed. Returns pairs of bytes, each pair encoding one DTC.

### Response Format
```
43 [count] [DTC1_hi] [DTC1_lo] [DTC2_hi] [DTC2_lo] ...
```

On non-CAN protocols, the response is padded to 6 data bytes (3 DTC slots).
On CAN protocols, an extra byte after `43` indicates the DTC count.

See [obd2_dtc_format.md](obd2_dtc_format.md) for DTC decoding.

## Service $04 - Clear/Reset DTCs

### Request Format
```
04
```

### Response
```
44     -- Success
```

### Side Effects (WARNING)
Issuing mode 04 erases ALL of the following:
- Number of stored DTCs
- All diagnostic trouble codes
- All freeze frame data
- DTC that initiated freeze frame
- All oxygen sensor test data
- Mode 06 and 07 information
- I/M readiness status

**Does NOT erase**: Permanent DTCs (mode 0A), which are only cleared by the ECU itself.

### Prerequisites
- Some vehicles require ignition ON, engine OFF
- SAE specifies scan tools must confirm ("Are you sure?") before sending
- ELM327 does NOT provide this confirmation - your software must

## Service $05 - O2 Sensor Monitoring

### Request Format
```
05 [Test ID] [O2 Sensor Number]
```

Returns oxygen sensor monitoring test results. Primarily used on older (non-CAN) vehicles. CAN vehicles use Service $06 instead.

## Service $06 - On-Board Monitoring Test Results

### Request Format
```
06 [Test ID]
```

Returns test results for on-board diagnostics monitoring (non-continuously monitored systems like catalyst, EVAP, etc.).

### Test ID Support Discovery
```
06 00        -- Get supported test IDs (00-1F)
```

## Service $07 - Pending DTCs

### Request Format
```
07
```

Same response format as Service $03, but returns DTCs detected during the current or last completed driving cycle that have not yet matured into stored (confirmed) DTCs.

Useful for verifying a repair without completing a full drive cycle.

## Service $08 - Control of On-Board Systems

### Request Format
```
08 [TID] [parameters]
```

Used by scan tools to command specific tests (e.g., EVAP leak test). Rarely used in general diagnostics. Implementation is manufacturer-specific.

## Service $09 - Vehicle Information

### Request Format
```
09 [InfoType]
```

### Common InfoTypes

| InfoType | Description | Typical Response Lines |
|----------|-------------|----------------------|
| `00` | Supported InfoTypes (bitmap) | 1 |
| `02` | Vehicle Identification Number (VIN) | 5 (17 ASCII chars) |
| `04` | Calibration ID | Variable |
| `06` | Calibration Verification Number (CVN) | Variable |
| `08` | In-Use Performance Tracking | Variable |
| `0A` | ECU Name | Variable |
| `0B` | In-Use Performance Tracking (compression ignition) | Variable |

### VIN Example
```
09 02 5          -- Request VIN, expect 5 lines
Response:
49 02 01 31 47 31
49 02 02 59 5A 32
49 02 03 33 36 37
49 02 04 30 39 38
49 02 05 37 30 36
```

Decode: Remove mode/PID/sequence bytes, convert hex to ASCII = 17-character VIN.

## Service $0A - Permanent DTCs

### Request Format
```
0A
```

Same response format as Service $03, but returns permanent DTCs. These:
- Cannot be cleared by a scan tool (mode 04)
- Cannot be cleared by disconnecting the battery
- Are only cleared by the ECU after it verifies the fault condition no longer exists
- Are required by US EPA regulations (2010+)

## Protocol-Specific Notes

### CAN vs Non-CAN Response Differences

**Non-CAN (J1850, ISO 9141, KWP)**:
- Service $03 response is padded to exactly 6 data bytes (3 DTC pairs)
- Data bytes are directly in the response

**CAN (ISO 15765-4)**:
- Service $03 adds an extra byte after the mode byte indicating DTC count
- Multiframe responses use ISO-TP segmentation
- Response IDs are physical (0x7E8-0x7EF)

### Timing Considerations
- J1850 (pre-2002): Minimum 100ms between requests
- J1850 (post-2002): Next request allowed immediately after all responses received
- CAN: No minimum interval required between requests
- ISO/KWP: Bus must be kept alive with wakeup messages (see [elm327_initialization.md](elm327_initialization.md))
