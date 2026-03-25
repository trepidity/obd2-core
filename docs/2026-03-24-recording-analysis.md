# Recording Analysis — Gap Findings

> **Purpose:** Ongoing analysis of real-world OBD-II recordings to identify gaps in obd2-core's spec, design, and implementation. New findings are appended as additional recordings are reviewed.

---

## Analysis 1: 2026-03-24 — STN2232 CAN Vehicle (Idle Session)

**Recordings reviewed:**
- `3070c852-16c1-4b16-864d-c1c06c131d9e.obd2raw` — 739s raw protocol capture, 20724 lines
- `f4895e28-4bb1-454f-a080-cfe700dc8edd.obd2rec` — matching binary recording, 4167 frames
- `sessions.json` — 6 sessions total (4 without VIN)

**Adapter:** STN2232 v5.10.3 (STN chip, CAN protocol, ~80ms round-trip per PID)

**Vehicle state:** Idling at ~773 RPM for entire 739-second session. Speed = 0 km/h throughout.

### Finding F1: Mode 05 on CAN Vehicles — 80% Bandwidth Waste [P0-Critical]

**Evidence:** The dashboard runs `read_all_o2_monitoring()` as a periodic task, issuing 72 Mode 05 commands (9 TIDs × 8 sensors) every ~13 seconds. **Every single one returns `NO DATA`** — 1440 failed requests across the session. Mode 05 is a non-CAN-only service (ISO 9141-2 / J1850).

**Measured impact:**
- Mode 05 scan time per cycle: ~10.4s
- Periodic cycle interval: ~13s
- Bandwidth wasted on Mode 05: **79.8%** of periodic scan budget
- RPM data blackouts: 12.3-second gaps every 13 seconds (e.g., 17.553s → 29.874s)
- Total time lost: ~589s of a 739s recording

**Root cause:** `modes.rs:read_all_o2_monitoring()` does not check `AdapterInfo.protocol` before issuing Mode 05. No protocol-awareness gating exists.

**Required fix:**
- Gate Mode 05 behind `!protocol.is_can()` check
- Auto-route to Mode 06 on CAN vehicles
- Or expose `capabilities.supports_mode05` so callers can decide

**Code refs:** `session/modes.rs:117-129`, `session/mod.rs:282-287`

---

### Finding F2: DTC Response Parsing — Count Byte Ambiguity [P0-Critical]

**Evidence:** Every Mode 03 response in the recording is `43 05 00 00 00 00 00`. After skipping the service echo byte (`43`), `decode_dtc_response()` receives `[05, 00, 00, 00, 00, 00]` and treats the pair `05 00` as DTC **P0500** (Vehicle Speed Sensor A).

**The problem:** On CAN protocols, the first byte after the service ID is a **DTC count byte**, not DTC data:
- **CAN format:** `43 [count] [DTC_hi] [DTC_lo] ...`
- **Legacy (non-CAN):** `43 [DTC_hi] [DTC_lo] ...` (no count byte, padded to 6 bytes)

If this is CAN (confirmed by ~80ms round-trip), then `05` = count of 5 DTCs, followed by zero data — meaning **no DTCs**. The current code generates a phantom P0500.

**Root cause:** `decode_dtc_response()` and `decode_dtc_bytes()` are protocol-unaware. `Elm327Adapter` knows the protocol from `ATDPN` but doesn't pass it to the DTC decoder.

**Required fix:** Protocol-aware DTC parsing that handles the count byte on CAN.

**Code refs:** `session/mod.rs:155-170`, `session/modes.rs:344-358`, `adapter/elm327.rs:267-273`

---

### Finding F3: No `STOPPED` / Command Interruption Handling [P1-High]

**Evidence:** `check_response_error()` at `elm327.rs:136` does not check for `STOPPED` responses. When an ELM327/STN command is interrupted (BLE disconnect, user cancel, NUL byte abort), the adapter returns `STOPPED\r\r>`.

**Impact:** `PollHandle::stop()` sets a cancellation flag but doesn't send an abort (NUL byte) to the adapter. The current in-flight command blocks until adapter timeout. No state recovery after interruption.

**Required fix:**
- Add `STOPPED` to `check_response_error()`
- Send NUL byte on cancel to abort in-flight command
- Add state recovery logic post-interruption

**Code refs:** `adapter/elm327.rs:136-168`, `session/poller.rs:90-92`

---

### Finding F4: No Multi-ECU Response Handling [P1-High]

**Evidence:** Adapter initializes with `ATH0` (headers off), which discards CAN source addresses. When multiple ECUs respond to a broadcast query, responses are merged with no source attribution.

**Impact on:**
- `supported_pids()` — can't distinguish ECM vs TCM supported PIDs
- DTC reads — can't attribute which module set a code
- Freeze frames — can't identify source ECU

**Required fix:**
- Add `ATH1` mode for operations where ECU attribution matters
- Parse CAN source addresses (7E8=ECM, 7E9=TCM, etc.)
- Per-ECU response demultiplexing

**Code refs:** `adapter/elm327.rs:206` (ATH0 during init), `adapter/elm327.rs:81-117` (parse_hex_response — no header handling)

---

### Finding F5: `NO DATA` Conflation — Unsupported vs Transient [P2-Medium]

**Evidence:** `poller.rs:186` treats every `Obd2Error::NoData` as "skip — PID not supported." But `NO DATA` means the ECU didn't respond within timeout, which can also mean: ECU busy, bus contention, adapter timeout too short, or intermittent wiring fault.

**Impact:** If a supported PID (listed in PID 00 bitmap) returns NO DATA transiently, it gets silently dropped forever. No retry, no backoff, no escalation.

**Required fix:** Distinguish "never supported" (not in bitmap) from "temporarily unavailable" (supported but NO DATA). Add retry/backoff for transient failures.

**Code refs:** `session/poller.rs:186-188`

---

### Finding F6: STN-Specific Commands Unused [P2-Medium]

**Evidence:** The adapter is STN2232 — detected as `Chipset::Stn` with full capabilities. But the code never uses STN-specific commands:
- `STPX` — protocol-level batch requests (send+receive in one transaction)
- `STMA` — CAN monitoring mode (passive, no polling overhead)
- `STF` — CAN filter configuration
- `STCMM` — CAN monitoring message mode

The recording shows serial one-at-a-time PID polling at ~12 PIDs/sec. STN batch mode could achieve 100+ PIDs/sec.

**Design note:** The design doc mentions `adapter/stn.rs` for STN extensions, but this file doesn't exist. All STN communication goes through the generic ELM327 path.

**Code refs:** `adapter/detect.rs:24-29` (capabilities set), `adapter/elm327.rs` (entire file — no STN paths)

---

### Finding F7: No Adaptive Scan Strategy [P2-Medium]

**Evidence from recording:**
- Vehicle speed = 0 km/h for 739 seconds (constant)
- STFT B1 (PID 06) = `0x80` = 0% for entire session (constant)
- LTFT B1 (PID 07) = `0x82` ≈ -1.6% for entire session (near-constant)
- Throttle (PID 11) = `0x19` = 9.8% for entire session (idle, constant)

These are polled at the same rate as RPM and engine load. No intelligence about:
- Which PIDs actually changed (delta-based suppression)
- Vehicle state (idle vs driving) for poll rate adaptation
- `PollHandle::set_interval()` exists but nothing uses it

**Code refs:** `session/poller.rs:94-97` (set_interval exists), `session/poller.rs:149-196` (fixed-rate cycle)

---

### Finding F8: Latency Not Captured in Reading API [P3-Low]

**Evidence from .obd2raw:**
- Write at t=X, first chunk at t=X+80ms, complete at t=X+128ms
- This timing data is gold for bus health diagnostics

**Impact:** `Reading.timestamp` is set to `Instant::now()` *after* parsing, not when the request was sent. Round-trip time is lost. `LoggingTransport` captures it to file, but it's not accessible through the API.

**Code refs:** `session/mod.rs:91-97` (timestamp set post-parse)

---

### Finding F9: `.obd2rec` Binary Format Has No Parser in obd2-core [P3-Low]

**Evidence:** 6 recordings exist in two format versions (`OBD2REC\x01` and `OBD2REC\x02`). Format stores pre-decoded f64 values with timestamps. No codec exists in obd2-core to read them.

**Structure observed:**
```
Magic:   "OBD2REC" (7 bytes)
Version: u8 (0x01 or 0x02)
Header:  u32 LE length + JSON metadata
Frames:  [type:u8] [timestamp:u32 LE ms] [pid:u8] [value:f64 LE] per frame (v1)
```

**Impact:** Recordings are opaque to the library. Can't replay, analyze, or re-decode with updated formulas. `store/mod.rs` defines storage traits but no `.obd2rec` codec.

---

### Finding F10: VIN-less Sessions — Silent Identification Failure [P3-Low]

**Evidence:** 4 of 6 sessions have `vin: null`, including `f4895e28` which captured 746 seconds of live data from a running vehicle. VIN read either wasn't attempted or failed silently.

**Impact:** Without VIN → no spec matching → no thresholds → no alerts → no vehicle context. The recording becomes much less useful for diagnostic replay.

---

### Observation O1: Voltage Readings Show Healthy Electrical System

ATRV readings across the session: 14.1V–14.4V (charging system active, alternator running). No sag events detected. This is a baseline for comparison with future recordings.

### Observation O2: Poll Cycle Timing Baseline

- 10 PIDs per cycle: 04, 05, 06, 07, 0B, 0C, 0D, 0E, 0F, 11
- Cycle time: ~1.28s (128ms per PID round-trip on CAN)
- Effective poll rate: ~0.78 Hz (without Mode 05 blackouts)
- Theoretical max with STN batch: ~8-10 Hz

### Observation O3: DTC Status Across Session

Mode 03 response consistent throughout: `43 05 00 00 00 00 00`. If correctly parsed (CAN count-byte interpretation), this is 0 stored DTCs. Needs protocol-aware verification.

---

## Summary Table

| ID | Finding | Priority | Status |
|----|---------|----------|--------|
| F1 | Mode 05 on CAN = 80% bandwidth waste | P0-Critical | Open |
| F2 | DTC count byte ambiguity (phantom P0500) | P0-Critical | Open |
| F3 | No STOPPED/interruption handling | P1-High | Open |
| F4 | No multi-ECU response handling | P1-High | Open |
| F5 | NO DATA conflation (unsupported vs transient) | P2-Medium | Open |
| F6 | STN-specific commands unused (10x throughput) | P2-Medium | Open |
| F7 | No adaptive scan strategy | P2-Medium | Open |
| F8 | Latency not in Reading API | P3-Low | Open |
| F9 | .obd2rec format has no parser | P3-Low | Open |
| F10 | VIN-less sessions (silent failure) | P3-Low | Open |
