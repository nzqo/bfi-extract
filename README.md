# Stuff

## Installing dependencies

Install pcap (todo link page)

```bash
pip install numpy maturin
```

## Build & Run

There are three projects in this workspace:

- `bfi_lib` contains common functionality to extract BFI-related stuff
  from pcap captures
- `bfi_cli` contains a small CLI to use some of that functionality from
  the command line
- `bfi_py_binding` contains a python binding to perform extraction from
  python

To build the CLI (which in turn builds the lib as dependency):

```bash
# cargo build --package bfi_cli
cargo run --package bfi_cli --features full_extract
```

To build the python binding, activate your venv and:

```bash
cd bfi_py_binding
maturin develop
```

Afterwards, the python binding will be installed in the venv.

## Testing

Run

```bash
cargo test
```
