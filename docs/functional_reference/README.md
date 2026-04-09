# OBD2 Functional Reference Library

Machine-readable reference documentation for OBD-II communications, ELM327 AT commands, and SAE J1979 standard PIDs. Designed for consumption by AI agents and automated tooling.

## Document Index

| File | Description |
|------|-------------|
| [elm327_at_commands.md](elm327_at_commands.md) | Complete ELM327 AT command reference grouped by category |
| [elm327_obd_protocols.md](elm327_obd_protocols.md) | OBD protocol definitions, IDs, baud rates, and selection logic |
| [elm327_programmable_parameters.md](elm327_programmable_parameters.md) | Programmable Parameter (PP) register definitions and values |
| [elm327_responses.md](elm327_responses.md) | Response formats, error messages, and status alerts |
| [elm327_initialization.md](elm327_initialization.md) | Bus initialization sequences, wakeup messages, and connection flow |
| [elm327_can_configuration.md](elm327_can_configuration.md) | CAN-specific configuration: filtering, masks, flow control, extended addressing |
| [elm327_j1939.md](elm327_j1939.md) | J1939 heavy-duty protocol support, PGN requests, and message formats |
| [sae_j1979_services.md](sae_j1979_services.md) | OBD-II service modes (01-0A) and request/response formats |
| [sae_j1979_pids.md](sae_j1979_pids.md) | Complete Service $01 PID definitions with signal decoding formulas |
| [obd2_dtc_format.md](obd2_dtc_format.md) | Diagnostic Trouble Code encoding, categories, and interpretation |
| [obd2_message_format.md](obd2_message_format.md) | OBD message structure: headers, data bytes, checksums across protocols |

## Sources

- **ELM327 Datasheet v2.0** (ELM327DSI) - Elm Electronics
- **ELM327 AT Commands Reference** - Elm Electronics, October 2010
- **OBDb/SAEJ1979** - CC-BY-SA-4.0 community signal set definitions
- **SAE J1979** / **ISO 15031-5** - OBD-II diagnostic services standard
- **SAE J1939** - Heavy-duty vehicle network standard

## Usage by AI Agents

Each document uses consistent structure:
- **YAML-style metadata blocks** at the top of tables for machine parsing
- **Hex values** are always prefixed or contextually obvious (e.g., `0x7E0`, `$01`)
- **Signal decoding formulas** use the format: `value = (raw * mul / div) + add`
- **Cross-references** between documents use `[file.md#section]` links
