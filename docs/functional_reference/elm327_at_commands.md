# ELM327 AT Command Reference

Complete AT command set for the ELM327 v2.0 OBD-to-RS232 interpreter IC.
All commands are prefixed with `AT` and terminated with a carriage return (`\r`, 0x0D).

## Command Format

```
AT <command> [arguments]\r
```

- Commands are case-insensitive
- Spaces are ignored
- Successful settings return `OK`
- Unknown commands return `?`
- A bare `\r` repeats the last command

---

## General Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `@1` | Display device description | - | v1.0 |
| `@2` | Display device identifier (12 chars, set with @3) | - | v1.3 |
| `@3 cccccccccccc` | Store device identifier (12 ASCII chars, 0x21-0x5F, write-once) | - | v1.3 |
| `D` | Set all to defaults (restores factory settings for current session) | - | v1.0 |
| `E0` | Echo off | - | v1.0 |
| `E1` | Echo on | * | v1.0 |
| `FE` | Forget events (clears ERR94/LV RESET CAN blocking flags) | - | v1.3a |
| `I` | Print version ID string (e.g., "ELM327 v2.0") | - | v1.0 |
| `L0` | Linefeeds off (CR only line termination) | - | v1.0 |
| `L1` | Linefeeds on (CR+LF line termination) | pin 7 | v1.0 |
| `LP` | Go to Low Power (standby) mode | - | v1.4 |
| `M0` | Memory off (don't save last protocol) | - | v1.0 |
| `M1` | Memory on (save last successful protocol) | pin 5 | v1.0 |
| `RD` | Read stored data byte | - | v1.4 |
| `SD hh` | Save data byte hh to EEPROM | - | v1.4 |
| `WS` | Warm start (software reset, retains some settings) | - | v1.0 |
| `Z` | Full reset (equivalent to power cycle) | - | v1.0 |

## RS232 / Baud Rate Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `BRD hh` | Try baud rate divisor hh (rate = 4000 / hh kbps) | - | v1.2 |
| `BRT hh` | Set baud rate handshake timeout to hh | - | v1.2 |

Baud rate divisor examples:

| Rate (kbps) | PP 0C Value |
|-------------|-------------|
| 9.6 | `00` |
| 19.2 | `D0` |
| 38.4 | `68` (default) |
| 57.6 | `45` |
| 115.2 | `23` |
| 230.4 | `11` |
| 500 | `08` |

## OBD General Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `AL` | Allow long (>7 byte) messages | - | v1.0 |
| `AR` | Automatic receive (restore default receive address) | - | v1.2 |
| `BD` | Perform buffer dump | - | v1.0 |
| `BI` | Bypass the initialization sequence | - | v1.0 |
| `DP` | Describe the current protocol (text name) | - | v1.0 |
| `DPN` | Describe the protocol by number (0-C) | - | v1.0 |
| `H0` | Headers off (hide header bytes in responses) | * | v1.0 |
| `H1` | Headers on (show header bytes in responses) | - | v1.0 |
| `MA` | Monitor all messages (all IDs, all protocols) | - | v1.0 |
| `MR hh` | Monitor for receiver address hh | - | v1.0 |
| `MT hh` | Monitor for transmitter address hh | - | v1.0 |
| `NL` | Normal length messages (7 bytes max) | * | v1.0 |
| `PC` | Protocol close (release bus) | - | v1.0 |
| `R0` | Responses off | - | v1.0 |
| `R1` | Responses on | * | v1.0 |
| `RA hh` | Set receive address to hh | - | v1.3 |
| `S0` | Printing of spaces off | - | v1.3 |
| `S1` | Printing of spaces on | * | v1.3 |
| `SH xx yy zz` | Set header bytes (3-byte form) | - | v1.0 |
| `SH yzz` | Set header bytes (CAN 11-bit form: priority + 11-bit ID) | - | v1.0 |
| `SH xx yy zz aa` | Set header bytes (CAN 29-bit form) | - | v1.0 |
| `SP h` | Set protocol to h and save to EEPROM | - | v1.0 |
| `SP Ah` | Set protocol to auto, starting with h, and save | - | v1.0 |
| `SP 00` | Set protocol to full auto search and save | - | v1.3 |
| `SR hh` | Set receive address to hh (alias for RA) | - | v1.2 |
| `SS` | Set standard (J1978) search order | - | v1.4 |
| `ST hh` | Set timeout to hh x 4 msec (max wait for response) | `32` | v1.0 |
| `TA hh` | Set tester address to hh | - | v1.4 |
| `TP h` | Try protocol h (no EEPROM write until success) | - | v1.0 |
| `TP Ah` | Try protocol h with auto search on failure | - | v1.0 |

## Adaptive Timing Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `AT0` | Adaptive timing off (use fixed ST value) | - | v1.2 |
| `AT1` | Adaptive timing auto1 (moderate adaptation) | * | v1.2 |
| `AT2` | Adaptive timing auto2 (aggressive adaptation) | - | v1.2 |

## Voltage Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `CV dddd` | Calibrate voltage to dd.dd volts | - | v1.0 |
| `CV 0000` | Restore voltage calibration to factory | - | v1.4 |
| `RV` | Read input voltage (returns "xx.xV") | - | v1.0 |

## ISO 9141 / ISO 14230 (KWP2000) Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `FI` | Perform fast initiation (protocol 5 only) | - | v1.4 |
| `IB 10` | Set ISO baud rate to 10400 | * | v1.0 |
| `IB 48` | Set ISO baud rate to 4800 | - | v1.4 |
| `IB 96` | Set ISO baud rate to 9600 | - | v1.0 |
| `IIA hh` | Set ISO (slow) init address to hh | `33` | v1.2 |
| `KW` | Display key words from last ISO init | - | v1.3 |
| `KW0` | Key word checking off | - | v1.2 |
| `KW1` | Key word checking on | * | v1.2 |
| `SI` | Perform slow initiation | - | v1.4 |
| `SW hh` | Set wakeup interval to hh x 20 msec | `92` | v1.0 |
| `WM [1-6 bytes]` | Set wakeup message content (header + data) | - | v1.0 |

Default wakeup messages:
- ISO 9141: `68 6A F1 01 00`
- ISO 14230 (KWP): `C1 33 F1 3E`

## J1850 Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `IFR0` | In-frame responses off | - | v1.2 |
| `IFR1` | In-frame responses auto | - | v1.2 |
| `IFR2` | In-frame responses on | - | v1.2 |
| `IFR H` | IFR value from header | * | v1.2 |
| `IFR S` | IFR value from source | - | v1.2 |

## CAN Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `CAF0` | CAN automatic formatting off | - | v1.0 |
| `CAF1` | CAN automatic formatting on | * | v1.0 |
| `CEA` | Turn off CAN extended addressing | * | v1.4 |
| `CEA hh` | Use CAN extended address hh | - | v1.4 |
| `CF hhh` | Set CAN ID filter (11-bit) | - | v1.0 |
| `CF hh hh hh hh` | Set CAN ID filter (29-bit) | - | v1.0 |
| `CFC0` | CAN flow control off | - | v1.0 |
| `CFC1` | CAN flow control on | * | v1.0 |
| `CM hhh` | Set CAN ID mask (11-bit) | - | v1.0 |
| `CM hh hh hh hh` | Set CAN ID mask (29-bit) | - | v1.0 |
| `CP hh` | Set CAN priority bits (29-bit only) | `18` | v1.0 |
| `CRA` | Reset CAN receive address filters | - | v1.4b |
| `CRA hhh` | Set CAN receive address (11-bit) | - | v1.3 |
| `CRA hhhhhhhh` | Set CAN receive address (29-bit) | - | v1.3 |
| `CS` | Show CAN status counts | - | v1.0 |
| `CSM0` | CAN silent mode off (ELM327 ACKs messages) | - | v1.4b |
| `CSM1` | CAN silent mode on (listen only, no ACKs) | * | v1.4b |
| `D0` | Display of DLC off | * | v1.3 |
| `D1` | Display of DLC on | - | v1.3 |
| `FC SD [1-5 bytes]` | Flow control: set data bytes | - | v1.1 |
| `FC SH hhh` | Flow control: set header (11-bit) | - | v1.1 |
| `FC SH hh hh hh hh` | Flow control: set header (29-bit) | - | v1.1 |
| `FC SM h` | Flow control: set mode (0=auto, 1=user-defined, 2=user+ID) | - | v1.1 |
| `PB xx yy` | Set protocol B parameters: xx=options, yy=baud divisor | - | v1.4 |
| `RTR` | Send RTR (remote transmission request) message | - | v1.3 |
| `V0` | Variable DLC off | * | v1.3 |
| `V1` | Variable DLC on | - | v1.3 |

## J1939 Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `DM1` | Monitor for DM1 (active DTCs) broadcast messages | - | v1.2 |
| `JE` | Use J1939 ELM data format (byte-reversed PGNs) | * | v1.3 |
| `JHF0` | J1939 header formatting off (show raw ID bytes) | - | v1.4b |
| `JHF1` | J1939 header formatting on (show priority + PGN + SA) | * | v1.4b |
| `JS` | Use J1939 SAE data format (standard byte order) | - | v1.3 |
| `JTM1` | J1939 timer multiplier 1x | * | v1.4b |
| `JTM5` | J1939 timer multiplier 5x (for slow multiline responses) | - | v1.4b |
| `MP hhhh` | Monitor for PGN 00hhhh | - | v1.2 |
| `MP hhhh n` | Monitor for PGN 00hhhh, get n messages | - | v1.4b |
| `MP hhhhhh` | Monitor for PGN hhhhhh | - | v1.3 |
| `MP hhhhhh n` | Monitor for PGN hhhhhh, get n messages | - | v1.4b |

## Programmable Parameter Commands

| Command | Description | Since |
|---------|-------------|-------|
| `PP xx SV yy` | Set value of programmable parameter xx to yy | v1.1 |
| `PP xx ON` | Enable programmable parameter xx | v1.1 |
| `PP xx OFF` | Disable programmable parameter xx | v1.1 |
| `PP FF ON` | Enable all programmable parameters | v1.1 |
| `PP FF OFF` | Disable all programmable parameters (factory reset) | v1.1 |
| `PPS` | Print programmable parameter summary | v1.1 |

## Other Commands

| Command | Description | Default | Since |
|---------|-------------|---------|-------|
| `IGN` | Read the IgnMon input level (returns ON or OFF) | - | v1.4 |
