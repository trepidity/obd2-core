# SAE J1979 Service $01 PID Reference

Complete Parameter ID definitions for OBD-II Service $01 (current data), including signal decoding formulas.
Sourced from OBDb/SAEJ1979 signal set (CC-BY-SA-4.0).

## Signal Decoding Formula

```
physical_value = (raw_value * mul / div) + add
```

Where:
- `raw_value` = unsigned integer from the response bytes
- `mul` = multiplier (default 1)
- `div` = divisor (default 1)
- `add` = offset (default 0)
- `len` = bit length of the raw value
- `bix` = bit index (starting position, 0 = MSB of first data byte)
- `sign` = true if value is signed (two's complement)

## CAN Addressing

Standard Service $01 request/response:
```
Request:  Header 7E0, Service 01, PID xx
Response: Header 7E8, Service 41, PID xx, data bytes
```

## PID Definitions

### PID $00 - Supported PIDs [01-20]
| Field | Value |
|-------|-------|
| Request | `01 00` |
| Response | 4 bytes, 32-bit bitmap |
| Frequency | Once at startup |
| Decoding | Each bit = 1 PID. Bit 0 (MSB) = PID $01, Bit 31 (LSB) = PID $20 |

### PID $01 - Monitor Status Since DTCs Cleared
| Field | Value |
|-------|-------|
| Request | `01 01` |
| Response | 4 bytes |
| Frequency | 10 Hz |

| Signal | Bit Index | Length | Unit | Description |
|--------|-----------|--------|------|-------------|
| MIL | 0 | 1 | on/off | Malfunction Indicator Lamp status |
| DTC_CNT | 1 | 7 | count | Number of confirmed emission DTCs |
| CCM_RDY | 9 | 1 | yes/no | Comprehensive component monitoring ready |
| FUEL_RDY | 10 | 1 | yes/no | Fuel system monitoring ready |
| MIS_RDY | 11 | 1 | yes/no | Misfire monitoring ready |
| CIM_SUP | 12 | 1 | map | 0=Spark ignition, 1=Compression ignition |
| CCM_SUP | 13 | 1 | yes/no | Comprehensive component monitoring supported |
| FUEL_SUP | 14 | 1 | yes/no | Fuel system monitoring supported |
| MIS_SUP | 15 | 1 | yes/no | Misfire monitoring supported |
| EGR_SUP | 16 | 1 | yes/no | EGR system monitoring supported |
| HTR_SUP | 17 | 1 | yes/no | O2 sensor heater monitoring supported |
| O2S_SUP | 18 | 1 | yes/no | O2 sensor monitoring supported |
| ACRF_SUP | 19 | 1 | yes/no | A/C refrigerant monitoring supported |
| AIR_SUP | 20 | 1 | yes/no | Secondary air system monitoring supported |
| EVAP_SUP | 21 | 1 | yes/no | Evaporative system monitoring supported |
| HCAT_SUP | 22 | 1 | yes/no | Heated catalyst monitoring supported |
| CAT_SUP | 23 | 1 | yes/no | Catalyst monitoring supported |
| EGR_RDY | 24 | 1 | yes/no | EGR system monitoring ready |
| HTR_RDY | 25 | 1 | yes/no | O2 sensor heater monitoring ready |
| O2S_RDY | 26 | 1 | yes/no | O2 sensor monitoring ready |
| ACRF_RDY | 27 | 1 | yes/no | A/C refrigerant monitoring ready |
| AIR_RDY | 28 | 1 | yes/no | Secondary air system monitoring ready |
| EVAP_RDY | 29 | 1 | yes/no | Evaporative system monitoring ready |
| HCAT_RDY | 30 | 1 | yes/no | Heated catalyst monitoring ready |
| CAT_RDY | 31 | 1 | yes/no | Catalyst monitoring ready |

### PID $02 - DTC That Caused Freeze Frame
| Field | Value |
|-------|-------|
| Request | `01 02` |
| Bytes | 2 |
| Unit | hex (DTC encoding) |
| Frequency | 60s |
| Note | 0x0000 = no freeze frame data |

### PID $03 - Fuel System Status
| Field | Value |
|-------|-------|
| Request | `01 03` |
| Bytes | 2 |
| Frequency | 0.25 Hz |

| Signal | Byte | Value Map |
|--------|------|-----------|
| FUELSYS1 | 1 | 0=OFF, 1=OL, 2=CL, 4=OL-Drive, 8=OL-Fault, 16=CL-Fault |
| FUELSYS2 | 2 | Same as above |

### PID $04 - Calculated Engine Load
| Field | Value |
|-------|-------|
| Request | `01 04` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $05 - Engine Coolant Temperature
| Field | Value |
|-------|-------|
| Request | `01 05` |
| Bytes | 1 |
| Formula | `value = raw - 40` |
| Unit | celsius |
| Range | -40 to 215 C |
| Frequency | 0.5 Hz |

### PID $06 - Short Term Fuel Trim (Bank 1)
| Field | Value |
|-------|-------|
| Request | `01 06` |
| Bytes | 1 |
| Formula | `value = (raw * 100 / 128) - 100` |
| Unit | percent |
| Range | -100 to 99.2% |
| Frequency | 0.25 Hz |

### PID $07 - Long Term Fuel Trim (Bank 1)
| Field | Value |
|-------|-------|
| Request | `01 07` |
| Bytes | 1 |
| Formula | `value = (raw * 100 / 128) - 100` |
| Unit | percent |
| Range | -100 to 99.2% |
| Frequency | 1 Hz |

### PID $0A - Fuel Pressure
| Field | Value |
|-------|-------|
| Request | `01 0A` |
| Bytes | 1 |
| Formula | `value = raw * 3` |
| Unit | kPa |
| Range | 0 - 765 kPa |
| Frequency | 1 Hz |

### PID $0B - Intake Manifold Absolute Pressure (MAP)
| Field | Value |
|-------|-------|
| Request | `01 0B` |
| Bytes | 1 |
| Formula | `value = raw` |
| Unit | kPa |
| Range | 0 - 255 kPa |
| Frequency | 1 Hz |

### PID $0C - Engine RPM
| Field | Value |
|-------|-------|
| Request | `01 0C` |
| Bytes | 2 |
| Formula | `value = raw / 4` |
| Unit | rpm |
| Range | 0 - 16383.75 rpm |
| Frequency | 0.25 Hz |

### PID $0D - Vehicle Speed
| Field | Value |
|-------|-------|
| Request | `01 0D` |
| Bytes | 1 |
| Formula | `value = raw` |
| Unit | km/h |
| Range | 0 - 255 km/h |
| Frequency | 0.25 Hz |

### PID $0E - Timing Advance
| Field | Value |
|-------|-------|
| Request | `01 0E` |
| Bytes | 1 |
| Formula | `value = (raw / 2) - 64` |
| Unit | degrees (before TDC) |
| Range | -64 to 63.5 degrees |
| Frequency | 0.25 Hz |

### PID $0F - Intake Air Temperature
| Field | Value |
|-------|-------|
| Request | `01 0F` |
| Bytes | 1 |
| Formula | `value = raw - 40` |
| Unit | celsius |
| Range | -40 to 215 C |
| Frequency | 0.25 Hz |

### PID $10 - MAF Air Flow Rate
| Field | Value |
|-------|-------|
| Request | `01 10` |
| Bytes | 2 |
| Formula | `value = raw / 100` |
| Unit | grams/sec |
| Range | 0 - 655.35 g/s |
| Frequency | 0.25 Hz |

### PID $11 - Throttle Position
| Field | Value |
|-------|-------|
| Request | `01 11` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $1C - OBD Standards Compliance
| Field | Value |
|-------|-------|
| Request | `01 1C` |
| Bytes | 1 |
| Frequency | 3600s |

| Value | Standard |
|-------|----------|
| 1 | OBD-II (California ARB) |
| 2 | OBD (Federal EPA) |
| 3 | OBD & OBD-II |
| 6 | EOBD (Europe) |
| 7 | EOBD & OBD-II |
| 10 | JOBD (Japan) |
| 14 | Euro IV B1 (Heavy Duty) |
| 17 | EMD (Engine Manufacturer Diagnostics) |
| 20 | HD OBD (Heavy Duty) |
| 21 | WWH OBD (World Wide Harmonized) |

### PID $1F - Runtime Since Engine Start
| Field | Value |
|-------|-------|
| Request | `01 1F` |
| Bytes | 2 |
| Formula | `value = raw` |
| Unit | seconds |
| Range | 0 - 65535 seconds |
| Frequency | 1 Hz |

### PID $21 - Distance Traveled with MIL On
| Field | Value |
|-------|-------|
| Request | `01 21` |
| Bytes | 2 |
| Formula | `value = raw` |
| Unit | km |
| Range | 0 - 65535 km |
| Frequency | 5 Hz |

### PID $2F - Fuel Tank Level
| Field | Value |
|-------|-------|
| Request | `01 2F` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 1 Hz |

### PID $31 - Distance Since DTCs Cleared
| Field | Value |
|-------|-------|
| Request | `01 31` |
| Bytes | 2 |
| Formula | `value = raw` |
| Unit | km |
| Range | 0 - 65535 km |
| Frequency | 15s |

### PID $33 - Barometric Pressure
| Field | Value |
|-------|-------|
| Request | `01 33` |
| Bytes | 1 |
| Formula | `value = raw` |
| Unit | kPa |
| Range | 0 - 255 kPa |
| Frequency | 0.25 Hz |

### PID $42 - Control Module Voltage
| Field | Value |
|-------|-------|
| Request | `01 42` |
| Bytes | 2 |
| Formula | `value = raw / 1000` |
| Unit | volts |
| Range | 0 - 65.535 V |
| Frequency | 10 Hz |

### PID $43 - Absolute Load Value
| Field | Value |
|-------|-------|
| Request | `01 43` |
| Bytes | 2 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 25700% |
| Frequency | 0.25 Hz |

### PID $44 - Commanded Equivalence Ratio (Lambda)
| Field | Value |
|-------|-------|
| Request | `01 44` |
| Bytes | 2 |
| Formula | `value = raw * 2 / 65535` |
| Unit | ratio |
| Range | 0 - 2 |
| Frequency | 0.25 Hz |

### PID $45 - Relative Throttle Position
| Field | Value |
|-------|-------|
| Request | `01 45` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $46 - Ambient Air Temperature
| Field | Value |
|-------|-------|
| Request | `01 46` |
| Bytes | 1 |
| Formula | `value = raw - 40` |
| Unit | celsius |
| Range | -40 to 215 C |
| Frequency | 15s |

### PID $47-$48 - Absolute Throttle Position B/C
| Field | Value |
|-------|-------|
| Request | `01 47` or `01 48` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $49-$4B - Accelerator Pedal Position D/E/F
| Field | Value |
|-------|-------|
| Request | `01 49`, `01 4A`, or `01 4B` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $4C - Commanded Throttle Actuator
| Field | Value |
|-------|-------|
| Request | `01 4C` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 0.25 Hz |

### PID $51 - Fuel Type
| Field | Value |
|-------|-------|
| Request | `01 51` |
| Bytes | 1 |
| Frequency | 0.25 Hz |

| Value | Fuel Type |
|-------|-----------|
| 1 | Gasoline/Petrol |
| 2 | Methanol |
| 3 | Ethanol |
| 4 | Diesel |
| 5 | LPG |
| 6 | CNG |
| 7 | Propane |
| 8 | Electric |
| 9-16 | Bi-fuel variants |
| 17-22 | Hybrid variants |
| 23 | Bi-fuel diesel |
| 29 | Fuel cell (hydrogen) |
| 30 | Hydrogen ICE |

### PID $5B - Hybrid Battery State of Charge
| Field | Value |
|-------|-------|
| Request | `01 5B` |
| Bytes | 1 |
| Formula | `value = raw * 100 / 255` |
| Unit | percent |
| Range | 0 - 100% |
| Frequency | 1 Hz |

### PID $5C - Engine Oil Temperature
| Field | Value |
|-------|-------|
| Request | `01 5C` |
| Bytes | 1 |
| Formula | `value = raw - 40` |
| Unit | celsius |
| Range | -40 to 210 C |
| Frequency | 1 Hz |

### PID $5E - Engine Fuel Rate
| Field | Value |
|-------|-------|
| Request | `01 5E` |
| Bytes | 2 |
| Formula | `value = raw / 20` |
| Unit | L/h |
| Range | 0 - 3212.75 L/h |
| Frequency | 0.25 Hz |

### PID $61 - Driver's Demand Engine Torque
| Field | Value |
|-------|-------|
| Request | `01 61` |
| Bytes | 1 |
| Formula | `value = raw - 125` |
| Unit | percent |
| Range | -125 to 130% |
| Frequency | 0.25 Hz |

### PID $62 - Actual Engine Torque
| Field | Value |
|-------|-------|
| Request | `01 62` |
| Bytes | 1 |
| Formula | `value = raw - 125` |
| Unit | percent |
| Range | -125 to 130% |
| Frequency | 0.25 Hz |

### PID $63 - Engine Reference Torque
| Field | Value |
|-------|-------|
| Request | `01 63` |
| Bytes | 2 |
| Formula | `value = raw` |
| Unit | Nm |
| Range | 0 - 65535 Nm |
| Frequency | 86400s |

### PID $A6 - Odometer
| Field | Value |
|-------|-------|
| Request | `01 A6` |
| Bytes | 4 |
| Formula | `value = raw / 10` |
| Unit | km |
| Range | 0 - 429,496,729.5 km |

### PID $B2 - Battery State of Health
| Field | Value |
|-------|-------|
| Request | `01 B2` |
| Bytes | 1 |
| Unit | percent |

### PID $D3 - Odometer (Engine Only)
| Field | Value |
|-------|-------|
| Request | `01 D3` |
| Bytes | 4 |
| Unit | km |

## Quick PID Formula Summary Table

| PID | Name | Bytes | Formula | Unit | Range |
|-----|------|-------|---------|------|-------|
| $04 | Engine Load | 1 | `A*100/255` | % | 0-100 |
| $05 | Coolant Temp | 1 | `A-40` | C | -40-215 |
| $06 | STFT Bank 1 | 1 | `(A*100/128)-100` | % | -100-99.2 |
| $07 | LTFT Bank 1 | 1 | `(A*100/128)-100` | % | -100-99.2 |
| $0A | Fuel Pressure | 1 | `A*3` | kPa | 0-765 |
| $0B | MAP | 1 | `A` | kPa | 0-255 |
| $0C | RPM | 2 | `((A*256)+B)/4` | rpm | 0-16383.75 |
| $0D | Speed | 1 | `A` | km/h | 0-255 |
| $0E | Timing Advance | 1 | `(A/2)-64` | deg | -64-63.5 |
| $0F | Intake Air Temp | 1 | `A-40` | C | -40-215 |
| $10 | MAF Rate | 2 | `((A*256)+B)/100` | g/s | 0-655.35 |
| $11 | Throttle Pos | 1 | `A*100/255` | % | 0-100 |
| $1F | Run Time | 2 | `(A*256)+B` | sec | 0-65535 |
| $2F | Fuel Level | 1 | `A*100/255` | % | 0-100 |
| $31 | Dist Since Clear | 2 | `(A*256)+B` | km | 0-65535 |
| $33 | Baro Pressure | 1 | `A` | kPa | 0-255 |
| $42 | Module Voltage | 2 | `((A*256)+B)/1000` | V | 0-65.535 |
| $44 | Lambda | 2 | `((A*256)+B)*2/65535` | ratio | 0-2 |
| $46 | Ambient Temp | 1 | `A-40` | C | -40-215 |
| $5C | Oil Temp | 1 | `A-40` | C | -40-210 |
| $5E | Fuel Rate | 2 | `((A*256)+B)/20` | L/h | 0-3212.75 |

Where `A` = first data byte, `B` = second data byte (after mode+PID bytes).
