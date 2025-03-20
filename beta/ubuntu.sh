#!/bin/bash
# Monero XMR Mining Setup for Ubuntu
# Usage: ./setup-monero-mining.sh YOUR_MONERO_WALLET_ADDRESS

# Check if wallet address was provided
if [ -z "$1" ]; then
  echo "Please provide your Monero wallet address as a parameter."
  echo "Usage: ./setup-monero-mining.sh YOUR_MONERO_WALLET_ADDRESS"
  exit 1
fi

WALLET_ADDRESS=$1
POOL_URL=${2:-"pool.supportxmr.com:3333"}
RIG_ID=${3:-"UbuntuMiner-$(hostname)"}

echo "===== MONERO MINING SETUP ====="
echo "Wallet address: $WALLET_ADDRESS"
echo "Pool URL: $POOL_URL"
echo "Rig ID: $RIG_ID"
echo "=============================="

# Update system and install dependencies
echo "[1/4] Updating system and installing dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential git cmake libuv1-dev libmicrohttpd-dev libssl-dev libhwloc-dev

# Clone XMRig repository
echo "[2/4] Downloading XMRig miner..."
mkdir -p ~/monero-mining
cd ~/monero-mining
git clone https://github.com/xmrig/xmrig.git
cd xmrig

# Build XMRig
echo "[3/4] Building XMRig miner..."
mkdir -p build
cd build
cmake ..
make -j$(nproc)

# Configure XMRig
echo "[4/4] Configuring XMRig for your wallet..."
cat > config.json << EOL
{
    "autosave": true,
    "cpu": true,
    "opencl": false,
    "cuda": false,
    "donate-level": 1,
    "pools": [
        {
            "url": "${POOL_URL}",
            "user": "${WALLET_ADDRESS}",
            "pass": "x",
            "keepalive": true,
            "tls": false,
            "rig-id": "${RIG_ID}"
        }
    ]
}
EOL

# Create convenience script to start mining
cat > ~/monero-mining/start-mining.sh << EOL
#!/bin/bash
cd ~/monero-mining/xmrig/build
./xmrig
EOL
chmod +x ~/monero-mining/start-mining.sh

# Set up systemd service (optional)
echo "Creating systemd service for auto-start..."
sudo tee /etc/systemd/system/monero-miner.service > /dev/null << EOL
[Unit]
Description=Monero XMR Miner
After=network.target

[Service]
Type=simple
User=$(whoami)
ExecStart=$(realpath ~/monero-mining/xmrig/build/xmrig)
WorkingDirectory=$(realpath ~/monero-mining/xmrig/build)
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOL

# Optimize system for mining
echo "Optimizing system for mining..."
if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
    echo "Setting CPU governor to performance mode (requires sudo)"
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo performance | sudo tee $cpu > /dev/null
    done
fi

echo "===== SETUP COMPLETE ====="
echo ""
echo "To start mining immediately:"
echo "  ~/monero-mining/start-mining.sh"
echo ""
echo "To enable mining as a service (starts automatically):"
echo "  sudo systemctl enable monero-miner.service"
echo "  sudo systemctl start monero-miner.service"
echo ""
echo "To check mining status when running as a service:"
echo "  sudo systemctl status monero-miner.service"
echo "  journalctl -u monero-miner.service -f"
echo ""
echo "Your mining statistics will be available at:"
echo "  https://supportxmr.com (search with your wallet address)"
echo ""
