# ELM327 Programmable Parameters

Non-volatile EEPROM settings that persist across power cycles. Each PP requires a two-step process to change: set the value, then enable it.

## Commands

```
AT PP xx SV yy   -- Set PP xx value to yy
AT PP xx ON      -- Enable PP xx
AT PP xx OFF     -- Disable PP xx
AT PP FF ON      -- Enable all PPs
AT PP FF OFF     -- Disable all PPs (factory reset)
AT PPS           -- Print PP summary table
```

## Effect Types

| Type | When Change Takes Effect |
|------|-------------------------|
| `I` | Immediately |
| `D` | After defaults restored (AT D, AT Z, AT WS, MCLR, power cycle) |
| `R` | After reset (AT Z, AT WS, MCLR, power cycle) |
| `P` | After power reset only (AT Z, MCLR, power cycle) |

## Emergency PP Reset

If you lose the ability to communicate (e.g., changed CR character or baud rate):
1. Connect a jumper between pin 28 (OBD Tx LED) and VSS (ground)
2. Power on the ELM327
3. Wait for RS232 Rx LED to flash rapidly
4. Remove the jumper - all PPs are now disabled

## Parameter Definitions

### PP 00 - Auto Monitor on Startup
| Field | Value |
|-------|-------|
| Default | `FF` (OFF) |
| Type | R |
| `00` | Perform AT MA after powerup |
| `FF` | Do not auto-monitor |

### PP 01 - Headers Display Default
| Field | Value |
|-------|-------|
| Default | `FF` (OFF = headers hidden) |
| Type | D |
| `00` | AT H1 (headers on) by default |
| `FF` | AT H0 (headers off) by default |

### PP 02 - Allow Long Messages Default
| Field | Value |
|-------|-------|
| Default | `FF` (OFF = normal 7-byte) |
| Type | D |
| `00` | AT AL by default |
| `FF` | AT NL by default |

### PP 03 - Response Timeout (AT ST default)
| Field | Value |
|-------|-------|
| Default | `32` (50 decimal = 205 msec) |
| Type | D |
| Formula | timeout = value x 4.096 msec |
| Range | `00`-`FF` (0 to ~1046 msec) |

### PP 04 - Adaptive Timing Default
| Field | Value |
|-------|-------|
| Default | `01` (AT1) |
| Type | D |
| `00` | AT0 (off) |
| `01` | AT1 (moderate) |
| `02` | AT2 (aggressive) |

### PP 06 - OBD Source (Tester) Address
| Field | Value |
|-------|-------|
| Default | `F1` |
| Type | R |
| Note | Not used for J1939 protocols |

### PP 07 - Last Protocol to Search
| Field | Value |
|-------|-------|
| Default | `09` |
| Type | I |
| Range | `01`-`0C` |

### PP 09 - Echo Default
| Field | Value |
|-------|-------|
| Default | `00` (ON) |
| Type | R |
| `00` | Echo on |
| `FF` | Echo off |

### PP 0A - Linefeed Character
| Field | Value |
|-------|-------|
| Default | `0A` (LF) |
| Type | R |

### PP 0C - RS232 Baud Rate Divisor
| Field | Value |
|-------|-------|
| Default | `68` (38400 baud) |
| Type | P |
| Formula | baud rate (kbps) = 4000 / value |
| Note | Only applies when pin 6 is high at startup |
| Special | `00` = 9600 bps |

### PP 0D - Carriage Return Character
| Field | Value |
|-------|-------|
| Default | `0D` (CR) |
| Type | R |
| Warning | Changing this may lock you out - use emergency reset if needed |

### PP 0E - Power Control Options
| Field | Value |
|-------|-------|
| Default | `9A` (10011010 binary) |
| Type | R |

| Bit | Function | 0 | 1 |
|-----|----------|---|---|
| b7 | Master enable | Off (legacy v1.0-v1.3a behavior) | On (enable LP functions) |
| b6 | Pin 16 full power level | Low | High |
| b5 | Auto LP on RS232 idle | Disabled | Enabled |
| b4 | Auto LP timeout | 5 minutes | 20 minutes |
| b3 | Auto LP warning | Disabled | Enabled (prints ACT ALERT) |
| b2 | IgnMon (pin 15) control | Disabled | Enabled (LP on ignition off) |
| b1 | IgnMon wake delay | 1 second | 5 seconds |
| b0 | Reserved | 0 | 0 |

### PP 0F - Activity Monitor Options
| Field | Value |
|-------|-------|
| Default | `D5` (11010101 binary) |
| Type | D |

| Bit | Function | 0 | 1 |
|-----|----------|---|---|
| b7 | Master control | Disabled | Enabled (allow b3-b6) |
| b6 | Wake from LP on OBD activity | No | Yes |
| b5 | Auto LP on OBD idle | Disabled | Enabled |
| b4 | OBD idle timeout | 30 seconds | 150 seconds |
| b3 | OBD idle warning | Disabled | Enabled (prints ACT ALERT) |
| b2 | Reserved | - | Leave at 1 |
| b1 | Exclamation prefix | No | Yes (adds ! before alerts) |
| b0 | LP LED flash | Disabled | Enabled (OBD Tx LED flashes in LP) |

### PP 10 - J1850 Voltage Settling Time
| Field | Value |
|-------|-------|
| Default | `0D` (53 msec) |
| Type | R |
| Formula | time = value x 4.096 msec |

### PP 11 - J1850 Break Signal Monitor
| Field | Value |
|-------|-------|
| Default | `00` (ON) |
| Type | D |
| `00` | Monitor enabled (reports BUS ERROR on break violations) |
| `FF` | Monitor disabled |

### PP 12 - J1850 Volts Pin Polarity
| Field | Value |
|-------|-------|
| Default | `FF` (normal) |
| Type | D |
| `00` | Inverted (high=5V, low=8V) |
| `FF` | Normal (low=5V, high=8V) |

### PP 13 - Auto Search Delay (Protocol 1 to 2)
| Field | Value |
|-------|-------|
| Default | `32` |
| Type | I |
| Formula | delay = value x 4.096 msec |

### PP 14 - ISO/KWP Stop Bit Width (P4 interbyte time)
| Field | Value |
|-------|-------|
| Default | `50` |
| Type | I |
| Formula | width = 98 + (value x 64) µsec |

### PP 15 - ISO/KWP Max Inter-byte Time (P1) and Min Inter-message Time (P2)
| Field | Value |
|-------|-------|
| Default | `0A` |
| Type | D |
| Formula | time = value x 2.112 msec |

### PP 16 - Default ISO/KWP Baud Rate
| Field | Value |
|-------|-------|
| Default | `FF` (10400 baud) |
| Type | R |
| `00` | 9600 baud |
| `FF` | 10400 baud |
| Note | 4800 baud cannot be a default; use AT IB 48 |

### PP 17 - ISO/KWP Wakeup Rate (AT SW default)
| Field | Value |
|-------|-------|
| Default | `92` (~3 seconds) |
| Type | D |
| Formula | interval = value x 20.48 msec |

### PP 18 - Auto Search Delay (Protocol 4 to 5)
| Field | Value |
|-------|-------|
| Default | `00` (no delay) |
| Type | I |
| Formula | delay = value x 4.096 msec |

### PP 19 - Delay After Protocol 5 (if 3&4 not tried)
| Field | Value |
|-------|-------|
| Default | `62` |
| Type | I |
| Formula | delay = value x 20.48 msec |

### PP 1A - Protocol 5 Fast Init Active Time (TiniL)
| Field | Value |
|-------|-------|
| Default | `0A` (25 msec) |
| Type | I |
| Formula | time = value x 2.5 msec |

### PP 1B - Protocol 5 Fast Init Passive Time (TiniH)
| Field | Value |
|-------|-------|
| Default | `0A` (25 msec) |
| Type | I |
| Formula | time = value x 2.5 msec |

### PP 1C - ISO/KWP Init Outputs
| Field | Value |
|-------|-------|
| Default | `03` |
| Type | I |
| Note | Controls which output lines (K, L) are used during initialization |

### PP 24 - CAN 4-byte Header Default
| Field | Value |
|-------|-------|
| Default | `00` |
| Type | D |

### PP 25 - CAN Auto-format Config
| Field | Value |
|-------|-------|
| Default | `00` |
| Type | D |

### PP 26 - CAN Filler Byte
| Field | Value |
|-------|-------|
| Default | `00` |
| Type | D |
| Note | Unused CAN data byte positions filled with this value (some systems use `AA` or `FF`) |

### PP 29 - CAN DLC Display Options
| Field | Value |
|-------|-------|
| Default | `FF` |
| Type | D |

### PP 2A - CAN Error Handling Options
| Field | Value |
|-------|-------|
| Default | `38` |
| Type | R |
| b5 | Block CAN search after ERR94 |
| b4 | Block CAN search after LV RESET |

### PP 2C - Protocol B (User1 CAN) Options
| Field | Value |
|-------|-------|
| Default | `E0` |
| Type | D |
| Note | Set to `42` for J1939 mode |

### PP 2D - Protocol B (User1 CAN) Baud Divisor
| Field | Value |
|-------|-------|
| Default | `04` (125 kbaud) |
| Type | D |
| Formula | baud = 500 / value kbaud |

### PP 2E - Protocol C (User2 CAN) Options
| Field | Value |
|-------|-------|
| Default | `80` |
| Type | D |

### PP 2F - Protocol C (User2 CAN) Baud Divisor
| Field | Value |
|-------|-------|
| Default | `0A` (50 kbaud) |
| Type | D |
| Formula | baud = 500 / value kbaud |
