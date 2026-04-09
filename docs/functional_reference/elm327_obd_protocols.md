# ELM327 OBD Protocol Reference

Supported OBD protocols, their characteristics, and selection mechanisms.

## Protocol Table

| ID | Name | Standard | Baud Rate | ID Bits | Bus Type |
|----|------|----------|-----------|---------|----------|
| `0` | Automatic | - | - | - | Auto-detect |
| `1` | SAE J1850 PWM | SAE J1850 | 41.6 kbaud | 3-byte header | Dual-wire differential |
| `2` | SAE J1850 VPW | SAE J1850 | 10.4 kbaud | 3-byte header | Single-wire |
| `3` | ISO 9141-2 | ISO 9141-2 | 10.4 kbaud (5 baud init) | 3-byte header | K-line (+ optional L-line) |
| `4` | ISO 14230-4 KWP (5 baud init) | ISO 14230-4 | 10.4 kbaud | 3-byte header | K-line |
| `5` | ISO 14230-4 KWP (fast init) | ISO 14230-4 | 10.4 kbaud | 3-byte header | K-line |
| `6` | ISO 15765-4 CAN (11 bit ID, 500 kbaud) | ISO 15765-4 | 500 kbaud | 11-bit CAN ID | CAN bus |
| `7` | ISO 15765-4 CAN (29 bit ID, 500 kbaud) | ISO 15765-4 | 500 kbaud | 29-bit CAN ID | CAN bus |
| `8` | ISO 15765-4 CAN (11 bit ID, 250 kbaud) | ISO 15765-4 | 250 kbaud | 11-bit CAN ID | CAN bus |
| `9` | ISO 15765-4 CAN (29 bit ID, 250 kbaud) | ISO 15765-4 | 250 kbaud | 29-bit CAN ID | CAN bus |
| `A` | SAE J1939 CAN | SAE J1939 | 250 kbaud* | 29-bit CAN ID | CAN bus |
| `B` | User1 CAN | User-defined | 125 kbaud* | 11-bit* CAN ID | CAN bus |
| `C` | User2 CAN | User-defined | 50 kbaud* | 11-bit* CAN ID | CAN bus |

`*` = user-adjustable via programmable parameters or AT PB command.

## Protocol Selection Commands

### Set Protocol (writes to EEPROM immediately)
```
AT SP h      -- Set protocol h as default
AT SP Ah     -- Set protocol h with auto-search fallback
AT SP 0      -- Full automatic search
AT SP 00     -- Same as SP 0
```

### Try Protocol (writes to EEPROM only on success)
```
AT TP h      -- Try protocol h only
AT TP Ah     -- Try protocol h, auto-search on failure
```

### Describe Protocol
```
AT DP        -- Returns text name (e.g., "ISO 15765-4 CAN (11 bit ID, 500 kbaud)")
AT DPN       -- Returns protocol number (e.g., "6")
```

## Automatic Search Behavior

When protocol 0 is selected (or auto-search is enabled with `A` prefix):

1. ELM327 sends `SEARCHING...` on first OBD command
2. Searches protocols based on active input detection (modern versions) or J1978 standard order (if `AT SS` was sent)
3. Uses **default OBD headers** during search (ignores user-defined headers)
4. Uses standard request `01 00` to test each protocol
5. On success, stores the protocol if memory is enabled (`AT M1`)
6. On total failure, returns `UNABLE TO CONNECT`

### Standard (J1978) Search Order
Protocols 1 through 9, in numeric order. Activated by `AT SS`.

### Modified Search Order (default)
The ELM327 inspects active input levels to prioritize likely protocols. CAN protocols (6-9) are typically tried first on modern vehicles.

### Search Limits
- PP 07 controls the last protocol number to try (default: `09`)
- ERR94 or LV RESET can block CAN searches (cleared by `AT FE` or power cycle)

## Protocol Characteristics

### J1850 PWM (Protocol 1)
- **Vehicles**: Ford (pre-2008)
- **Bus pins**: J1962 pin 2 (Bus+) and pin 10 (Bus-)
- **Voltage**: 5V differential
- **Message**: 3-byte header + up to 7 data bytes + CRC
- **Requires ACK**: Yes (ELM327 acknowledges received messages)

### J1850 VPW (Protocol 2)
- **Vehicles**: GM (pre-2008)
- **Bus pins**: J1962 pin 2 (Bus+)
- **Voltage**: 0V / 8V single-ended
- **Message**: 3-byte header + up to 7 data bytes + CRC
- **Requires ACK**: No

### ISO 9141-2 (Protocol 3)
- **Vehicles**: Chrysler, European, Asian (older)
- **Bus pins**: J1962 pin 7 (K-line), optional pin 15 (L-line)
- **Init**: 5-baud slow initialization (2-3 seconds)
- **Wakeup**: Required every 5 seconds (`AT SW` controls interval)
- **Message**: 3-byte header + up to 7 data bytes + checksum
- **Default wakeup**: `68 6A F1 01 00`

### ISO 14230-4 KWP2000 (Protocols 4 and 5)
- **Vehicles**: European, Asian (2003-2008 era)
- **Bus pins**: J1962 pin 7 (K-line)
- **Init**: 5-baud (protocol 4) or fast init (protocol 5, ~300ms)
- **Wakeup**: Required every 5 seconds
- **Message**: 3-byte header + up to 7 data bytes + checksum
- **Default wakeup**: `C1 33 F1 3E`
- **Key words**: Available via `AT KW` after successful init

### ISO 15765-4 CAN (Protocols 6-9)
- **Vehicles**: All US vehicles 2008+, many 2004+
- **Bus pins**: J1962 pin 6 (CAN-H), pin 14 (CAN-L)
- **Transceiver**: Required (e.g., MCP2551, MCP2562)
- **Message format**: CAN ID + PCI byte + data (auto-formatted by default)
- **Max data**: 8 bytes per frame, multiframe via ISO-TP (up to 4095 bytes)
- **Silent mode**: Default on (`AT CSM1`), ELM327 does not ACK

### SAE J1939 (Protocol A)
- **Vehicles**: Heavy-duty trucks, buses, agricultural/construction equipment
- **Bus**: CAN 250 kbaud, 29-bit IDs
- **Addressing**: Source address in ID bits, destination or broadcast
- **Message format**: PGN-based (Parameter Group Numbers)
- **See**: [elm327_j1939.md](elm327_j1939.md) for full details

### User CAN (Protocols B and C)
- **Purpose**: Custom/experimental CAN configurations
- **Configuration**: Via PP 2C-2F or `AT PB xx yy`
- **Options byte**: Controls 11/29-bit ID, variable DLC, formatting
- **Baud divisor**: `500 / desired_kbaud` (e.g., 01 for 500 kbaud)

## J1962 OBD-II Connector Pinout

```
Pin  Function
 1   Manufacturer discretionary
 2   J1850 Bus+ (protocols 1, 2)
 3   Manufacturer discretionary
 4   Chassis ground
 5   Signal ground
 6   CAN-H (protocols 6-9, A-C)
 7   K-line (protocols 3, 4, 5)
 8   Manufacturer discretionary
 9   Manufacturer discretionary
10   J1850 Bus- (protocol 1 only)
11   Manufacturer discretionary
12   Manufacturer discretionary
13   Manufacturer discretionary
14   CAN-L (protocols 6-9, A-C)
15   L-line (protocol 3, optional)
16   Battery positive (+12V)
```

## Default OBD Headers by Protocol

| Protocol | Default Header | Meaning |
|----------|---------------|---------|
| 1 (J1850 PWM) | `61 6A F1` | Priority=6, Functional addr, Tester=F1 |
| 2 (J1850 VPW) | `68 6A F1` | Priority=6, Functional addr, Tester=F1 |
| 3 (ISO 9141) | `68 6A F1` | Standard OBD request |
| 4, 5 (KWP) | `C0 33 F1` | KWP functional request |
| 6 (CAN 11/500) | `7DF` | Functional broadcast ID |
| 7 (CAN 29/500) | `18 DB 33 F1` | Functional broadcast |
| 8 (CAN 11/250) | `7DF` | Functional broadcast ID |
| 9 (CAN 29/250) | `18 DB 33 F1` | Functional broadcast |
| A (J1939) | `18 EA FF F9` | Priority 6, request to all, from F9 |
