# OBD-II Message Format Reference

Low-level message structure across all OBD-II protocols.

## General Message Structure

All OBD protocols follow the same conceptual pattern:

```
[Header/ID] [Data Bytes] [Checksum/CRC]
```

The ELM327 handles headers and checksums automatically. With `AT H0` (default), only data bytes are shown. With `AT H1`, headers are visible.

## J1850 Message Format (Protocols 1 & 2)

```
┌──────────┬──────────┬──────────┬─────────────────┬──────────┐
│ Priority │ Target   │ Source   │ Data (1-7 bytes) │ CRC/Chk  │
│ (1 byte) │ (1 byte) │ (1 byte) │                  │ (1 byte) │
└──────────┴──────────┴──────────┴─────────────────┴──────────┘
```

### Header Byte Meanings

**Priority byte** (J1850):
```
Bits 7-5: Priority (0=highest, 7=lowest)
Bit 4:    Header type (0=3-byte, 1=1-byte)
Bit 3:    In-frame response (0=not required, 1=required)
Bit 2:    Addressing mode (0=functional, 1=physical)
Bits 1-0: Message type
```

**Common Headers**:
| Header Bytes | Meaning |
|-------------|---------|
| `68 6A F1` | Functional request from tester (F1) |
| `61 6A F1` | J1850 PWM functional request |
| `48 6B 10` | Physical response from ECU at address 10 |

### J1850 PWM (Protocol 1)
- Differential signaling: Bus+ and Bus-
- 5V levels
- CRC checksum
- Requires acknowledgement (IFR)
- Ford vehicles primarily

### J1850 VPW (Protocol 2)
- Single-wire: Bus+ only
- 0V/8V levels
- CRC checksum
- No acknowledgement required
- GM vehicles primarily

## ISO 9141-2 / ISO 14230-4 Message Format (Protocols 3, 4, 5)

```
┌──────────┬──────────┬──────────┬─────────────────┬──────────┐
│ Format   │ Target   │ Source   │ Data (1-7 bytes) │ Checksum │
│ (1 byte) │ (1 byte) │ (1 byte) │                  │ (1 byte) │
└──────────┴──────────┴──────────┴─────────────────┴──────────┘
```

### Format Byte (ISO 14230-4 / KWP2000)
```
Bits 7-6: Address mode
  00 = no address info (physical with header)
  01 = CARB mode (functional with 1-byte target)
  10 = physical addressing with header bytes
  11 = functional addressing

Bits 5-0: Data length (0-63 bytes)
  If 00, an additional length byte follows the header
```

**Common Headers**:
| Header Bytes | Protocol | Meaning |
|-------------|----------|---------|
| `68 6A F1` | ISO 9141 | Standard OBD request |
| `C0 33 F1` | ISO 14230 | KWP functional request |
| `C1 33 F1` | ISO 14230 | KWP functional request (1 data byte) |
| `48 6B 10` | Both | Response from ECU address 0x10 |

### Checksum Calculation
Simple 8-bit sum of all bytes (header + data), modulo 256:
```
checksum = (sum of all bytes) & 0xFF
```

The ELM327 calculates and verifies checksums automatically.

### Key Addresses
| Address | Device |
|---------|--------|
| `33` | Default ECU (functional) |
| `F1` | Scan tool / tester |
| `10` | Engine ECU (typical) |
| `18` | Transmission ECU (typical) |
| `28` | ABS ECU (typical) |

## ISO 15765-4 CAN Message Format (Protocols 6-9)

```
┌─────────────┬──────────────────────────────┐
│  CAN ID     │  CAN Data Field (8 bytes)    │
│ (11 or 29b) │  [PCI] [OBD Data] [Padding]  │
└─────────────┴──────────────────────────────┘
```

### CAN ID Assignments

**11-bit IDs**:
| ID Range | Purpose |
|----------|---------|
| `7DF` | Functional request (broadcast to all ECUs) |
| `7E0-7E7` | Physical request to specific ECU |
| `7E8-7EF` | Physical response from specific ECU |

ECU address mapping:
```
Request  -> Response
7E0      -> 7E8      (ECU #1, typically engine)
7E1      -> 7E9      (ECU #2, typically transmission)
7E2      -> 7EA      (ECU #3)
...
7E7      -> 7EF      (ECU #8)
```

**29-bit IDs**:
| ID | Purpose |
|----|---------|
| `18 DB 33 F1` | Functional request |
| `18 DA F1 xx` | Physical response (xx = ECU address) |
| `18 DA xx F1` | Physical request to ECU xx |

### PCI (Protocol Control Information) Byte

The first byte of the CAN data field indicates the frame type:

#### Single Frame (SF) - PCI type 0
```
[0L] [data bytes] [padding]
 │
 └── L = data length (1-7)

Example: 02 01 00 00 00 00 00 00
         │  └─ Data: mode 01, PID 00
         └──── SF, 2 data bytes
```

#### First Frame (FF) - PCI type 1
```
[1L] [LL] [data bytes]
 │    │
 │    └── Total message length (low byte)
 └─────── 1 + high nibble of length

Total length = ((first_byte & 0x0F) << 8) | second_byte

Example: 10 14 49 02 01 31 47 31
         │  │  └── First 6 data bytes
         │  └───── Total = 0x014 = 20 bytes
         └──────── First Frame
```

#### Consecutive Frame (CF) - PCI type 2
```
[2N] [data bytes]
 │
 └── N = sequence number (1-F, wraps to 0)

Example: 21 59 5A 32 33 36 37 30
         │  └── 7 data bytes
         └───── CF, sequence #1
```

#### Flow Control (FC) - PCI type 3
```
[3S] [BS] [STmin]
 │    │    │
 │    │    └── Minimum separation time between CFs (ms)
 │    └─────── Block size (0 = send all remaining)
 └──────────── S: 0=ContinueToSend, 1=Wait, 2=Overflow

Example: 30 00 00
         │  │  └── No minimum delay
         │  └───── Send all frames
         └──────── Continue to send
```

### Multiframe Example

Request VIN (17 characters = requires multi-frame):
```
Request:   7DF: 02 09 02 00 00 00 00 00

Response:  7E8: 10 14 49 02 01 31 47 31   (FF: 20 bytes total)
Tester->:  7E0: 30 00 00 00 00 00 00 00   (FC: send all)
           7E8: 21 59 5A 32 33 36 37 30   (CF seq 1)
           7E8: 22 39 38 37 30 36 00 00   (CF seq 2)
```

### CAN Auto Formatting (AT CAF1)

When enabled (default), the ELM327:
- **Sending**: Automatically adds PCI bytes, calculates length, pads data
- **Receiving**: Strips PCI bytes, reassembles multiframe messages, shows only OBD data
- **Display**: Shows clean OBD response (e.g., `41 0C 1A F8`)

When disabled (AT CAF0):
- Raw CAN data shown including PCI bytes
- No multiframe reassembly
- You must construct complete frames manually

## Response Timing

### Timeout (AT ST)
```
AT ST hh     -- Set timeout to hh x 4.096 ms

Default: 0x32 = 50 x 4.096 = ~205 ms
Maximum: 0xFF = 255 x 4.096 = ~1045 ms
```

### Adaptive Timing
| Mode | Behavior |
|------|----------|
| AT0 | Fixed timeout (AT ST value) |
| AT1 | Moderate adaptation (default) |
| AT2 | Aggressive adaptation (faster for known-good connections) |

### Expected Response Count
Append a single hex digit to any OBD request:
```
01 0C 1     -- Get RPM, expect 1 response (skip remaining timeout)
09 02 5     -- Get VIN, expect 5 lines
03 1        -- Get DTCs, expect 1 responding ECU
```

## Data Byte Order

All OBD-II data is transmitted **big-endian** (most significant byte first).

For multi-byte values:
```
Response: 41 0C 1A F8
PID 0C (RPM): bytes = 1A F8
Decimal: (0x1A * 256 + 0xF8) = 6904
Apply formula: 6904 / 4 = 1726 RPM
```

## Protocol Comparison Summary

| Feature | J1850 PWM | J1850 VPW | ISO 9141 | KWP2000 | CAN |
|---------|-----------|-----------|----------|---------|-----|
| Speed | 41.6k | 10.4k | 10.4k | 10.4k | 250k/500k |
| Header size | 3 bytes | 3 bytes | 3 bytes | 3 bytes | 11/29 bits |
| Max data | 7 bytes | 7 bytes | 7 bytes | 7 bytes | 4095 bytes (ISO-TP) |
| Error check | CRC | CRC | Sum | Sum | CAN CRC |
| Init required | No | No | Yes (slow) | Yes (slow/fast) | No |
| Keepalive | No | No | Yes (5s) | Yes (5s) | No |
| ACK required | Yes | No | No | No | CAN-level |
| Bus wires | 2 (diff) | 1 | 1-2 (K, L) | 1 (K) | 2 (CAN-H, CAN-L) |
