# ELM327 Response Formats and Error Messages

How the ELM327 communicates results, errors, and status alerts.

## Standard Responses

| Response | Meaning |
|----------|---------|
| `OK` | AT command executed successfully |
| `?` | Command not understood (syntax error, invalid AT command, or inappropriate command for current protocol) |
| `>` | Prompt character - IC is idle, ready for next command |
| `SEARCHING...` | Auto protocol search in progress |
| `ELM327 v2.0` | Identification string (on reset) |

## OBD Response Format

### Normal Response (headers off)
```
41 0C 1A F8
│  │  └────── Data bytes
│  └───────── PID echoed
└──────────── Mode + 0x40 (response indicator)
```

### Normal Response (headers on, non-CAN)
```
48 6B 10 41 0C 1A F8 C4
│  │  │  │  │  └────── Data
│  │  │  │  └───────── PID
│  │  │  └──────────── Response mode (01+40)
│  │  └─────────────── Source address (ECU)
│  └────────────────── Target address
└───────────────────── Priority
                                    └── Checksum
```

### Normal Response (headers on, CAN 11-bit)
```
7E8 06 41 0C 1A F8 00 00
│   │  │  │  └────────── Data + padding
│   │  │  └──────────────PID
│   │  └─────────────── Response mode
│   └────────────────── PCI byte (length)
└────────────────────── CAN ID (ECU response)
```

### Response Mode Calculation
```
Response mode = Request mode + 0x40

Mode 01 request  ->  41 response
Mode 02 request  ->  42 response
Mode 03 request  ->  43 response
Mode 09 request  ->  49 response
```

### Number of Expected Responses
Append a single hex digit to any OBD request to limit responses:
```
01 05 1    -- Get ECT, expect exactly 1 response (faster)
09 02 5    -- Get VIN, expect 5 lines
```

## Error Messages

### `?`
Command not recognized. Causes:
- Typing error in AT command
- Invalid hex characters in OBD command
- Command not applicable to current protocol (e.g., `AT FI` when not on protocol 5)
- Incomplete command (20-second timeout)

### `NO DATA`
No response received within the AT ST timeout period. Causes:
- Vehicle has no data for the requested PID
- Mode not supported by ECU
- Ignition not on
- CAN filter blocking the response
- Wrong protocol selected

**Fix**: Increase `AT ST` value, check protocol, verify CAN filters (`AT CRA`).

### `UNABLE TO CONNECT`
Automatic protocol search tried all protocols and found none. Causes:
- Ignition key not in ON position
- Vehicle uses unsupported protocol
- Wiring problem
- Wrong connector or non-OBD-II vehicle

### `BUS BUSY`
Too much bus activity to insert a message. Causes:
- Wiring problem giving continuously active input (most common)
- Bus genuinely saturated (rare)

### `BUS ERROR`
Invalid signal detected on bus. Causes:
- Pulse duration exceeds protocol limits
- Wiring error
- Normal during some vehicle startups (if monitoring all)

### `CAN ERROR`
CAN initialization, send, or receive failure. Causes:
- Not connected to a CAN bus
- Wrong protocol or baud rate
- Wiring problem in CAN interface

### `DATA ERROR`
Response received but data was incorrect or unrecoverable.

### `<DATA ERROR`
Error in the specific response line indicated. Causes:
- Incorrect checksum
- Message format error
- Noise interference
- CAN auto-formatting enabled for non-ISO 15765-4 system

### `BUFFER FULL`
Internal 512-byte RS232 transmit buffer overflow. Causes:
- Data arriving faster than PC can read
- Low baud rate

**Fix**: Increase baud rate, use `AT H0` and `AT S0` to reduce output, apply CAN filters.

### `<RX ERROR`
CAN data receive error. Causes:
- Wrong baud rate for the bus being monitored
- Unacknowledged messages on bus
- Bit errors in CAN frame

### `FB ERROR`
Feedback error - output was energized but no corresponding input detected. Almost always a wiring problem in initial circuit builds.

### `ERRxx`
Internal error with two-digit code. Contact Elm Electronics for interpretation.

**ERR94** (special): Fatal CAN error requiring full IC reset.
- All settings return to defaults
- Blocks further automatic CAN searches until `AT FE` or power cycle
- Usually caused by CAN wiring problems or non-CAN signals on pins 6/14

### `LV RESET`
Low voltage (brownout) reset triggered. Causes:
- VDD dropped below ~2.8V
- Sudden large VDD change
- CAN transceiver drawing excessive current

Equivalent to AT Z reset. Blocks CAN auto-search until `AT FE` or next reset.

## Alert Messages

### `ACT ALERT` / `!ACT ALERT`
No RS232 or OBD activity for extended period. Low power mode will engage unless activity resumes within 1 minute (RS232) or immediately (OBD timeout). The `!` prefix appears when PP 0F bit 1 is set.

### `LP ALERT` / `!LP ALERT`
Low power mode will engage in 2 seconds. Cannot be stopped once initiated. This is the final warning before standby.

### `BUS INIT: ...`
ISO 9141/14230 bus initialization in progress (only shown when auto-search is off).
- Three dots indicate slow init (2-3 seconds)
- No dots for fast init
- Followed by `OK` or error message

### `STOPPED`
OBD operation was interrupted by:
- RS232 character received during processing
- Low level on RTS pin (pin 15)

Usually means the host sent a new command before waiting for the `>` prompt.

## NULL Bytes Warning

The ELM327 may occasionally insert NULL bytes (0x00) into RS232 output due to a known EUSART errata in the underlying PIC18F2480. Software should:
- Filter/ignore incoming 0x00 bytes
- Use "hide control characters" in terminal programs
