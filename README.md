# BfiExtract

A small library to extract information out of the compressed beamforming
feedback angles (BFAs) captured from channel sounding session.

This workspace contains:

- `bfi_lib` The library implementing the core functionality
- `bfi_cli` A mini application to perform extraction from the command line
- `bfi_py_binding` A python binding to directly extract information into numpy arrays

## Installing dependencies

Install pcap (todo link page)

```bash
pip install numpy maturin
```

## Build & Run

To build the CLI (which in turn builds the lib as dependency):

```bash
cargo build --package bfi_cli
cargo run --package bfi_cli 
```

## Python Binding

To build the python binding, install maturin and use it to install
the package in your virtual environment:

```bash
# Activate venv from whereever first, then:
cd bfi_py_binding
maturin develop
```

Afterwards, the python binding will be installed in the venv.

## Testing

To run some unit tests, just use cargo:

```bash
cargo test
```
