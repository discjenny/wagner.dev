# Wagner.dev Deployment

## Update Project on Server

```bash
cd /opt/wagner.dev
git pull
cargo build --release -j 1
sudo systemctl restart wagner-dev
sudo systemctl status wagner-dev
```

## View Logs
```bash
sudo journalctl -u wagner-dev -f
``` 