# OBD-II Diagnostic Trouble Code (DTC) Format

How DTCs are encoded in OBD-II responses and how to decode them into standard 5-character codes.

## DTC Structure

Each DTC is encoded as 2 bytes (16 bits):

```
Byte 1:  [C1 C0] [B B B B B B]
Byte 2:  [B B B B B B B B]

C1,C0 = Category (2 bits)
B...B = Code digits (14 bits, interpreted as 3.5 hex digits)
```

### Full Decode

```
Bits [15:14] = Category prefix (2 bits)
Bits [13:12] = Second character (2 bits -> 0-3)
Bits [11:8]  = Third character (4 bits -> 0-F hex)
Bits [7:4]   = Fourth character (4 bits -> 0-F hex)
Bits [3:0]   = Fifth character (4 bits -> 0-F hex)
```

## Category Prefix Lookup

| First Hex Digit | Replace With | Category | Authority |
|-----------------|-------------|----------|-----------|
| `0` | `P0` | Powertrain | SAE/ISO defined |
| `1` | `P1` | Powertrain | Manufacturer defined |
| `2` | `P2` | Powertrain | SAE/ISO defined |
| `3` | `P3` | Powertrain | SAE/ISO jointly defined |
| `4` | `C0` | Chassis | SAE/ISO defined |
| `5` | `C1` | Chassis | Manufacturer defined |
| `6` | `C2` | Chassis | Manufacturer defined |
| `7` | `C3` | Chassis | Reserved for future |
| `8` | `B0` | Body | SAE/ISO defined |
| `9` | `B1` | Body | Manufacturer defined |
| `A` | `B2` | Body | Manufacturer defined |
| `B` | `B3` | Body | Reserved for future |
| `C` | `U0` | Network/Communication | SAE/ISO defined |
| `D` | `U1` | Network/Communication | Manufacturer defined |
| `E` | `U2` | Network/Communication | Manufacturer defined |
| `F` | `U3` | Network/Communication | Reserved for future |

## Decoding Examples

### Example 1: `01 33`
```
Hex bytes: 01 33
First hex digit: 0 -> P0
Remaining: 133
DTC = P0133 (Oxygen Sensor Circuit Slow Response, Bank 1, Sensor 1)
```

### Example 2: `D0 16`
```
Hex bytes: D0 16
First hex digit: D -> U1
Remaining: 016
DTC = U1016 (Network Communication)
```

### Example 3: `11 31`
```
Hex bytes: 11 31
First hex digit: 1 -> P1
Remaining: 131
DTC = P1131 (Manufacturer-specific powertrain code)
```

### Example 4: `00 00`
```
Not a real DTC - this is padding. Ignore DTC pairs of 0000.
```

## Service $03 Response Decoding

### Non-CAN Response
```
43 01 33 00 00 00 00
│  └─┬─┘ └─┬─┘ └─┬─┘
│   DTC1   DTC2   DTC3 (padding)
└── Mode 03 response

DTC1 = 0133 -> P0133
DTC2 = 0000 -> padding (ignore)
DTC3 = 0000 -> padding (ignore)
```

### CAN Response
```
43 02 01 33 04 20 00 00
│  │  └─┬─┘ └─┬─┘
│  │   DTC1   DTC2
│  └── Count: 2 DTCs follow
└──── Mode 03 response

DTC1 = 0133 -> P0133
DTC2 = 0420 -> P0420 (Catalyst System Efficiency Below Threshold)
```

## Determining DTC Count (Service $01, PID $01)

```
Request:  01 01
Response: 41 01 81 07 65 04

Third byte: 81 (hex) = 10000001 (binary)
  Bit 7 (MSB): 1 = MIL (Check Engine Light) is ON
  Bits 6-0:    0000001 = 1 DTC stored

If MIL is on, subtract 0x80 from the byte to get DTC count:
  0x81 - 0x80 = 0x01 = 1 DTC
```

## DTC Status Byte (Enhanced Diagnostics)

Some protocols provide additional status information per DTC:

| Bit | Meaning |
|-----|---------|
| 0 | Test failed at time of request |
| 1 | Test failed during current driving cycle |
| 2 | Pending DTC |
| 3 | Confirmed DTC |
| 4 | Test not completed since last clear |
| 5 | Test failed since last clear |
| 6 | Test not completed this driving cycle |
| 7 | Warning indicator requested |

## DTC Categories

### Powertrain (P-codes)
- **P0xxx**: SAE/ISO standard codes (emissions-related)
- **P1xxx**: Manufacturer-specific
- **P2xxx**: SAE/ISO standard codes (extended)
- **P3xxx**: SAE/ISO + manufacturer jointly defined

### Chassis (C-codes)
- **C0xxx**: SAE/ISO standard
- **C1xxx-C2xxx**: Manufacturer-specific
- **C3xxx**: Reserved

### Body (B-codes)
- **B0xxx**: SAE/ISO standard
- **B1xxx-B2xxx**: Manufacturer-specific
- **B3xxx**: Reserved

### Network (U-codes)
- **U0xxx**: SAE/ISO standard (communication faults)
- **U1xxx-U2xxx**: Manufacturer-specific
- **U3xxx**: Reserved

## Common P0 Codes

| DTC | Description |
|-----|-------------|
| P0100-P0104 | Mass Air Flow circuit |
| P0105-P0109 | Manifold Absolute Pressure circuit |
| P0110-P0114 | Intake Air Temperature circuit |
| P0115-P0119 | Engine Coolant Temperature circuit |
| P0120-P0124 | Throttle Position Sensor circuit |
| P0130-P0167 | Oxygen Sensor circuits |
| P0170-P0175 | Fuel Trim issues |
| P0200-P0212 | Injector circuit issues |
| P0300 | Random/Multiple Cylinder Misfire |
| P0301-P0312 | Cylinder X Misfire Detected |
| P0335-P0339 | Crankshaft Position Sensor |
| P0340-P0344 | Camshaft Position Sensor |
| P0400-P0409 | EGR system |
| P0410-P0419 | Secondary Air system |
| P0420 | Catalyst System Efficiency Below Threshold (Bank 1) |
| P0430 | Catalyst System Efficiency Below Threshold (Bank 2) |
| P0440-P0457 | EVAP system |
| P0500-P0504 | Vehicle Speed Sensor |
| P0505-P0509 | Idle Control System |
| P0600-P0606 | Internal Control Module |
