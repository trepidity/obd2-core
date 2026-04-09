# ELM327 CAN Configuration Reference

CAN-specific settings for filtering, masking, flow control, extended addressing, and message formatting.

## CAN ID Formats

### 11-bit Standard CAN ID
```
Bits: [10:0] = 11-bit identifier
Range: 0x000 to 0x7FF

OBD functional request:  0x7DF (broadcast to all ECUs)
OBD physical request:    0x7E0 - 0x7E7 (specific ECU)
OBD physical response:   0x7E8 - 0x7EF (request ID + 0x08)
```

### 29-bit Extended CAN ID
```
Bits: [28:0] = 29-bit identifier
Structured as 4 bytes: PP FF DD SS

PP = Priority + data page (typically 0x18)
FF = PDU Format
DD = Destination address (or group extension)
SS = Source address

OBD functional request:  18 DB 33 F1
OBD physical response:   18 DA F1 xx (where xx = ECU address)
```

## CAN Receive Filtering

### CRA Command (Simple Filter)
The fastest way to filter CAN responses:

```
AT CRA hhh          -- Accept only 11-bit ID hhh
AT CRA hhhhhhhh     -- Accept only 29-bit ID hhhhhhhh
AT CRA              -- Reset all filters (accept everything)
```

Examples:
```
AT CRA 7E8          -- Only accept ECU #1 responses
AT CRA 18DAF110     -- Only accept responses from ECU at address 0x10
```

### Mask and Filter (Advanced)
For pattern matching on ID bits:

```
AT CM hhh            -- Set 11-bit mask
AT CF hhh            -- Set 11-bit filter
AT CM hh hh hh hh   -- Set 29-bit mask
AT CF hh hh hh hh   -- Set 29-bit filter
```

**Logic**: A message is accepted when:
```
(received_ID & mask) == (filter & mask)
```

Example - Accept all IDs 0x7E0 through 0x7EF:
```
AT CM 7F0       -- Mask: care about upper bits, ignore lower nibble
AT CF 7E0       -- Filter: must match 0x7Ex
```

Example - Accept IDs 0x7E8 and 0x7E9 only:
```
AT CM 7FE       -- Mask: ignore only bit 0
AT CF 7E8       -- Filter: match 0x7E8 or 0x7E9
```

## CAN Auto Formatting (ISO 15765-4)

### AT CAF1 (Default: On)
When enabled, the ELM327:
- Automatically adds PCI (Protocol Control Information) bytes to outgoing messages
- Strips PCI bytes from incoming responses
- Handles ISO-TP multiframe reassembly
- Pads unused data bytes with the filler byte (PP 26, default 0x00)

### AT CAF0 (Off)
Raw CAN mode:
- You must manually construct complete CAN data frames
- PCI bytes are shown in responses
- No multiframe reassembly
- Useful for non-standard CAN protocols

## CAN Flow Control

ISO 15765-4 multiframe transfers require flow control negotiation.

### AT CFC1 (Default: On)
ELM327 automatically handles flow control:
- Sends Flow Control (FC) frames when receiving multiframe data
- Processes FC frames when sending multiframe data

### AT CFC0 (Off)
Disables automatic flow control. Use for monitoring or non-standard protocols.

### Custom Flow Control
For ECUs that require non-standard flow control:

```
AT FC SM h              -- Set flow control mode
                        --   0 = auto (default values)
                        --   1 = user-defined header and data
                        --   2 = user-defined header, auto data

AT FC SH hhh            -- Set flow control header (11-bit)
AT FC SH hh hh hh hh   -- Set flow control header (29-bit)
AT FC SD [1-5 bytes]    -- Set flow control data bytes
```

Example - Custom flow control for a specific ECU:
```
AT FC SH 7E0            -- FC header = 7E0
AT FC SD 30 00 00       -- FC data: ContinueToSend, no delay, all frames
AT FC SM 1              -- Use user-defined FC
```

### ISO-TP PCI Byte Types
| PCI Type | Nibble | Description |
|----------|--------|-------------|
| Single Frame (SF) | `0x` | x = data length (1-7) |
| First Frame (FF) | `1x` | Start of multiframe, x+next byte = total length |
| Consecutive Frame (CF) | `2x` | x = sequence number (1-F, wraps to 0) |
| Flow Control (FC) | `3x` | x = flow status (0=CTS, 1=Wait, 2=Overflow) |

## CAN Extended Addressing

Some ECUs use an extra addressing byte within the CAN data field:

```
AT CEA hh    -- Enable extended addressing with byte hh
AT CEA       -- Disable extended addressing
```

When enabled:
- First data byte of every frame is the extended address
- The ELM327 inserts/strips this automatically
- Effectively reduces usable data per frame by 1 byte

## CAN Silent Mode

```
AT CSM1      -- Silent mode on (default): listen only, no ACKs
AT CSM0      -- Silent mode off: participate in bus (send ACKs)
```

**Important**: When bench-testing with a single CAN node, `AT CSM0` is required to prevent `<RX ERROR` messages caused by unacknowledged frames.

## CAN Status

```
AT CS        -- Show CAN status counts
```

Returns transmit and receive error counters from the CAN controller.

## DLC (Data Length Code) Display

```
AT D0        -- DLC display off (default)
AT D1        -- DLC display on (shows byte count before data)
```

## Variable DLC

```
AT V0        -- Variable DLC off (default): always send 8 bytes
AT V1        -- Variable DLC on: send only the number of data bytes needed
```

## CAN Priority (29-bit only)

```
AT CP hh     -- Set CAN priority bits
```

Default is `18` (priority 6 with DP=0, EDP=0):
```
Binary: 11000 (3 priority bits = 110 = 6, EDP=0, DP=0)
```

## Protocol B and C Configuration

User CAN protocols B and C are configured via:

```
AT PB xx yy  -- Set protocol B: xx=options byte, yy=baud divisor
```

Or via Programmable Parameters:
- PP 2C / PP 2E: Options byte
- PP 2D / PP 2F: Baud rate divisor (500 / divisor = kbaud)

### Options Byte Bit Definitions

| Bit | Function |
|-----|----------|
| b7 | ID length: 0=11-bit, 1=29-bit |
| b6 | Data formatting: 0=none, 1=ISO 15765-4 |
| b5 | Use separate Rx address: 0=no, 1=yes |
| b4-b0 | Reserved / protocol-specific |

For J1939 on protocol B: options = `42`, divisor = `02` (250 kbaud).

## CAN Filler Byte

Unused data positions in CAN frames are padded:
```
PP 26 = 00    -- Default: pad with 0x00
PP 26 = AA    -- Alternative: pad with 0xAA
PP 26 = FF    -- Alternative: pad with 0xFF
```

## RTR (Remote Transmission Request)

```
AT RTR       -- Send an RTR message
```

Sends a CAN frame with the RTR bit set, using the current header/ID. Some ECUs respond to RTR requests with data.

## Common CAN Configurations

### Standard OBD-II CAN (most modern cars)
```
AT SP 6              -- or AT SP 7 for 29-bit
AT CRA 7E8           -- Filter for ECU #1
AT SH 7E0            -- Address ECU #1
```

### Monitor All CAN Traffic
```
AT SP 6              -- Select CAN protocol
AT CSM0              -- Participate in bus
AT H1                -- Show headers
AT MA                -- Monitor all
```

### Custom Baud Rate CAN
```
AT PB E0 02          -- Protocol B: 11-bit, 250 kbaud
AT SP B              -- Select protocol B
```
