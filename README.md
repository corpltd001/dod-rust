# DOD Miners

## From source
### Build
install rustup

https://www.rust-lang.org/tools/install

```bash
cargo build --release
```


### Run

#### Bash run
```bash
./target/release/dod_miner miner --threads=$cpu_threads --cycles_price=$cycles_price --wif=$wif_priv_key
```

eg.
```bash
./target/release/dod_miner miner --threads=12 --cycles_price=0.5 --wif=xxxxxxxxxxxxxxxxxxxxx
```

## Binary
Download the binary from relases based on the platform
unzip the file
On windows, use powershell
```bash
./dod_miner miner --threads=$cpu_threads --cycles_price=$cycles_price --wif=$wif_priv_key
```
eg.
```bash
./dod_miner miner --threads=12 --cycles_price=0.5 --wif=xxxxxxxxxxxxxxxxxxxxx
```