# Cinergy

Cinergy is a framework designed to model the **isolated power consumption** of virtual resources (such as VMs). By "isolated," we mean that, unlike other energy models, our goal is to determine the power consumption a resource would exhibit if it were the only one in the system.

This component serves as the agent that exposes estimated power consumption and carbon emissions for QEMU-based VMs. By loading a previously generated polynomial model, it tracks the CPU usage of VMs on the host through procfs and translates this data into power and carbon readings available on demand through a REST API.

## Power Model

The power estimation uses the **cinergy ratio** to separate static and dynamic contributions:

```
static_power  = f(0) × (vm_cores / server_cores)
dynamic_power = (f(cpu_usage) - f(0)) × cinergy_ratio
total_power   = static_power + dynamic_power
```

Where:
- **f(x)** is the polynomial model loaded from `REG_COEFF` (e.g. `f(x) = c₀ + c₁x + c₂x² + ...`)
- **f(0)** represents the server's static (idle) power consumption
- **vm_cores / server_cores** allocates a proportional share of static power to the VM
- **cinergy_ratio** (between 0 and 1) scales the dynamic power contribution
- **vm_cores** are auto-detected from the QEMU process command line (`-smp` argument)

## Carbon Emissions

Carbon emissions are derived from the power estimate:

```
carbon_intensity (gCO2eq/h) = (power_watts / 1000) × DC_PUE × EMISSION_FACTOR
```

Where:
- **DC_PUE** is the datacenter Power Usage Effectiveness (≥ 1.0)
- **EMISSION_FACTOR** is the grid carbon intensity in gCO2eq/kWh

## Features

- **CPU Usage Tracking**: Periodically monitors the CPU usage of all running QEMU-based VMs via procfs.
- **vCPU Detection**: Automatically detects the number of virtual CPUs assigned to each VM.
- **Polynomial Estimation**: Uses a polynomial model with cinergy ratio to estimate isolated VM power consumption.
- **Carbon Emissions**: Computes carbon emission rate from power estimate, PUE, and grid emission factor.
- **REST API**: Offers GET endpoints for power estimation and carbon emissions per VM.

## Configuration

All configuration is loaded from a `.env` file at startup:

| Variable          | Description                                       | Example      |
|-------------------|---------------------------------------------------|--------------|
| `REG_COEFF`       | Comma-separated polynomial coefficients           | `1.0,2.0,3.0`|
| `PORT`            | HTTP server port (default: 8080)                  | `9999`       |
| `CINERGY_RATIO`   | Dynamic power scaling factor (0.0 to 1.0)         | `0.75`       |
| `SERVER_CORES`    | Total number of physical cores on the host         | `16`         |
| `DC_PUE`          | Datacenter Power Usage Effectiveness (≥ 1.0)       | `1.3`        |
| `EMISSION_FACTOR` | Grid carbon intensity in gCO2eq/kWh (≥ 0.0)       | `50.0`       |

## Setup

```bash
git clone https://github.com/yourusername/cinergy-prototype.git
cd cinergy-prototype
cp dotenv .env
# Edit .env with your model coefficients, server info, and carbon parameters
cargo build
cargo run
```

## API

### `GET /power?vm={name}`

Returns the estimated power consumption breakdown for the specified VM.

**Response (200):**
```json
{
  "vm": "my-vm",
  "cpu_usage": 1.5,
  "vm_cores": 4,
  "server_cores": 16,
  "cinergy_ratio": 0.75,
  "static_power_watts": 2.5,
  "dynamic_power_watts": 11.25,
  "total_power_watts": 13.75
}
```

### `GET /carbon?vm={name}`

Returns the estimated carbon emission rate for the specified VM.

**Response (200):**
```json
{
  "vm": "my-vm",
  "power_watts": 13.75,
  "pue": 1.3,
  "emission_factor_gco2_per_kwh": 50.0,
  "carbon_intensity_gco2_per_hour": 0.89375
}
```

### Error responses

| Status | Condition                          |
|--------|------------------------------------|
| 400    | Empty `vm` parameter               |
| 404    | No CPU usage data tracked for VM   |

## Testing

```bash
cargo test
```

Unit tests cover:
- Polynomial model evaluation
- Cinergy ratio power estimation formula (static/dynamic split)
- VM name and vCPU count parsing from QEMU command lines
- Carbon intensity calculation

## Architecture

```
src/
├── main.rs              # Entry point: loads config, starts tracker, runs server
├── config.rs            # AppConfig struct loaded from .env
├── model/
│   └── mod.rs           # PolynomialModel with cinergy ratio power estimation
├── monitor/
│   ├── mod.rs           # CpuUsageTracker: background CPU monitoring
│   └── cpu_time.rs      # procfs interaction, QEMU process discovery
└── endpoint/
    ├── mod.rs
    └── server.rs        # Actix-web REST handlers and server setup
```

## License

BSD 3-Clause. See [LICENSE](LICENSE).
