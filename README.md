# tailwind
```bash
cargo run -p tailwind
```

# development
```bash
cargo run -p dev
```

# deployment

```bash
cd /opt/wagner.dev
git pull
cargo build --release -j 1
sudo systemctl restart wagner-dev
sudo systemctl status wagner-dev
```

## logs
```bash
sudo journalctl -u wagner-dev -f
``` 