# Cinergy

Cinergy is a framework designed to model the isolated power consumption of virtual resources (such as VMs). By "isolated," we mean that, unlike other energy models, our goal is to determine the power consumption a resource would exhibit if it were the only one in the system.

This component serves as the agent that exposes estimated power consumption to VMs. By loading a previously generated model, it tracks the CPU usage of QEMU-based VMs on the host and translates this data into power readings available on demand through a REST endpoint.

## Features

- **CPU Usage Tracking**: Periodically monitors the CPU usage of all running QEMU-based VMs.
- **Polynomial Estimation**: Uses a polynomial model to estimate CPU usage.
- **REST API**: Offers a GET endpoint that accepts a VM name and returns the estimated power usage based on the tracked data.

## Setup

```bash
git clone https://github.com/yourusername/vm-cpu-monitor.git
cd cinergy-prototype
cp dotenv .env
cargo build
cargo run
```
Remember to edit the ```.env``` file to specify the generated formula (polynomial coefficients) and the server port.

## Usage

```bash
curl "http://127.0.0.1:9999/power?vm=name"
```
