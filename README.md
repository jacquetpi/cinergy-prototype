# Cinergy prototype

Cinergy is a framework aiming to model the isolated power consumption of virtual resources (such as VMs). By isolated, we mean that, unlike others energy models, we want to find the power consumption a resource would have if it was alone in the system

This is the agent exposing to VMs their estimated power consumption.
By loading a model (previously generated), it tracks the CPU usage of the host QEMU VMs and translate it to power readings on demand (through a REST endpoint).

## Features

- **Track CPU Usage**: Periodically monitors the CPU usage of all running QEMU-based VMs.
- **Polynomial Estimation**: Uses a polynomial model to estimate the CPU usage.
- **REST API**: Provides a GET endpoint that takes a VM name and returns the estimated power usage based on the tracked data.

## Setup

```bash
git clone https://github.com/yourusername/vm-cpu-monitor.git
cd cinergy-prototype
cp dotenv .env
cargo build
cargo run
```

Don't forget to modify the ```.env``` file to specify the generated formula (through polynomial coefficients) and the port

## Usage

```bash
curl "http://127.0.0.1:9999/power?vm=name"
```