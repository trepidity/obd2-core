# ELM327 Initialization and Connection Flow

Startup sequences, bus initialization, wakeup messages, and connection lifecycle.

## Power-On Sequence

1. ELM327 powers up and reads Programmable Parameters from EEPROM
2. Checks pin 28 for PP emergency reset jumper
3. Reads pin 5 (Memory default), pin 6 (Baud rate), pin 7 (Linefeed mode)
4. LED lamp test: all four LEDs flash in sequence
5. Sends identification string: `ELM327 v2.0\r\r>`
6. If PP 00 is enabled, performs `AT MA` (monitor all)
7. Enters idle state, waiting for commands

## RS232 Communication Setup

```
Default:   9600 baud  (pin 6 = low at powerup)
Alternate: 38400 baud (pin 6 = high at powerup, or as set by PP 0C)

Settings: 8 data bits, no parity, 1 stop bit (8N1)
```

### First Connection Test
```
Send:  AT Z\r
Expect: ELM327 v2.0\r\r>
```

## Typical OBD Connection Sequence

### Automatic Protocol Detection
```
AT Z          -- Full reset
AT E0         -- Echo off (optional, reduces traffic)
AT SP 0       -- Set to automatic protocol search
01 00         -- Request supported PIDs (triggers search)
              -- ELM327 responds with "SEARCHING..." then data or error
AT DPN        -- Check which protocol was found
```

### Known Protocol Connection
```
AT Z          -- Full reset
AT E0         -- Echo off
AT SP 6       -- Set to CAN 11-bit 500k (or known protocol)
01 00         -- Verify connection
```

### Optimized Fast Connection
```
AT Z          -- Full reset
AT E0         -- Echo off
AT S0         -- Spaces off (reduces bytes)
AT H0         -- Headers off
AT L0         -- Linefeeds off
AT CAF1       -- CAN auto formatting on
AT ST 32      -- Set timeout to 200ms
AT AT1        -- Adaptive timing on
AT SP 6       -- Set protocol
01 00 1       -- Request with expected response count
```

## ISO 9141-2 Bus Initialization (Protocol 3)

### Slow Init Sequence (5-baud)
1. ELM327 waits for bus to be idle
2. Sends 5-baud address byte (default `33`, changeable via `AT IIA hh`)
3. Waits for sync pattern from ECU (~200ms)
4. Receives two key word bytes from ECU
5. Sends inverted second key word as confirmation
6. ECU sends inverted address byte as final ACK
7. Bus is now initialized

**Duration**: 2-3 seconds
**Display**: `BUS INIT: ...OK` (or error)

### Key Words
After successful init, key words available via `AT KW`.

## ISO 14230-4 KWP2000 Initialization

### Slow Init (Protocol 4)
Same as ISO 9141-2 slow init above.

### Fast Init (Protocol 5)
1. ELM327 holds K-line low for TiniL (default 25ms, PP 1A)
2. Releases K-line high for TiniH (default 25ms, PP 1B)
3. Sends StartCommunication request at 10400 baud
4. ECU responds with StartCommunication positive response
5. Bus is now initialized

**Duration**: ~300ms
**Manual trigger**: `AT FI` (fast init) or `AT SI` (slow init)

## CAN Initialization (Protocols 6-9)

CAN protocols require no initialization handshake. The ELM327:
1. Configures CAN controller for the selected baud rate and ID size
2. Enters silent monitoring mode by default (`AT CSM1`)
3. Sends the first OBD request
4. Waits for response within AT ST timeout

### CAN Silent Mode
- Default ON: ELM327 only listens, does not ACK frames
- Turn OFF with `AT CSM0` for proper bus participation
- Must be OFF for bench testing with a single node (otherwise `<RX ERROR`)

## Wakeup / Keep-Alive Messages

ISO 9141 and ISO 14230 buses require periodic activity to stay awake.

### Automatic Wakeup
- ELM327 sends wakeup messages automatically when idle
- Default interval: ~3 seconds (PP 17, adjustable via `AT SW hh`)
- Must be within the 5-second bus timeout

### Default Wakeup Content
| Protocol | Default Message |
|----------|----------------|
| ISO 9141 | `68 6A F1 01 00` |
| ISO 14230 (KWP) | `C1 33 F1 3E` |

### Custom Wakeup Message
```
AT WM hh hh hh hh [hh hh]   -- 1 to 6 bytes (checksum added automatically)
```

Example:
```
AT WM 11 22 33 44 55   -- Custom wakeup: header=11 22 33, data=44 55
```

## Low Power Mode

### Entering Low Power
Three methods:

1. **AT LP command**: Immediate (after 1-second delay and LP ALERT)
2. **RS232 idle timeout**: After 5 or 20 minutes of no RS232 activity (PP 0E b5, b4)
3. **IgnMon (pin 15) low**: When ignition is turned off (PP 0E b7 and b2 both = 1)

### Power Consumption
| State | Typical Current |
|-------|----------------|
| Normal operation | ~12 mA (IC only) |
| Low power standby | ~0.15 mA (IC only) |

### Waking from Low Power
Three methods:

1. **RS232 Rx pulse**: Low pulse >128 µsec on pin 18 (send a space or @ at ≤57.6 kbps)
2. **OBD activity**: Any active level on OBD input pins (requires all-quiet first)
3. **IgnMon (pin 15) high**: Ignition restored (after 1 or 5 second delay per PP 0E b1)

### Settings Retained After Wake
These settings survive the low power -> wake transition:
```
E0/E1    H0/H1    L0/L1    M0/M1
R0/R1    D0/D1    S0/S1    AT0/AT1/AT2
CAF0/1   CFC0/1   CSM0/1   CEA
JTM1/5   AL/NL    IIA      Current protocol (reset but not changed)
```

## Warm Start vs Full Reset

| Feature | AT WS (Warm Start) | AT Z (Full Reset) |
|---------|--------------------|--------------------|
| LED test | No | Yes |
| ID string | No | Yes (prints "ELM327 v2.0") |
| Restore defaults | Yes | Yes |
| Read PPs | Yes | Yes |
| EEPROM write | No | No |
| Speed | Fast | Slower |

## Connection Timeout Behavior

The AT ST command controls how long the ELM327 waits for a response:

```
AT ST hh    -- timeout = hh x 4.096 msec
```

| Value | Timeout |
|-------|---------|
| `01` | ~4 ms |
| `19` | ~102 ms |
| `32` | ~205 ms (default) |
| `64` | ~410 ms |
| `FF` | ~1046 ms |

With adaptive timing (AT1/AT2), the ELM327 adjusts this automatically based on observed response times.

### J1939 Special Timing
J1939 automatically uses extended timeouts (~1.25 seconds). The JTM5 command multiplies this by 5x for slow multiline responses.
