# ledsim


Simulator for:

https://github.com/n0ctu/LEDs-get-crazy-payloads

## Running

Pass with Port, Height and Width

```bash
cargo run --release --package ledsim -- 54321 24 48 20
```

## Use

```bash
cat /dev/urandom | nc -u 127.0.0.1 54321
```

## Release Build

```bash
./target/release/ledsim
```

