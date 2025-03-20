#!/bin/sh
# Complete Monero Setup Guide for Alpine Linux

echo "===== STEP 1: INSTALLING DEPENDENCIES ====="
# Update system and install dependencies
apk update
apk add build-base git libmicrohttpd-dev hwloc-dev openssl-dev cmake libuv-dev

# For Monero wallet
apk add boost-dev openssl-dev readline-dev unbound-dev curl-dev

echo "===== STEP 2: BUILDING MONERO WALLET ====="
# Clone and build Monero from source (Alpine doesn't have a prebuilt package)
cd ~
git clone --recursive https://github.com/monero-project/monero.git
cd monero
make -j$(nproc)

echo "===== STEP 3: CREATING A MONERO WALLET ====="
# Create directory for wallet
mkdir -p ~/monero-wallet
cd ~/monero-wallet

# Generate new wallet (CLI method)
echo "Creating a new Monero wallet. Follow the prompts to set a wallet name and password."
echo "IMPORTANT: Write down your seed phrase! It's your only backup."
~/monero/build/release/bin/monero-wallet-cli --generate-new-wallet ~/monero-wallet/mywallet

echo "===== STEP 4: GETTING YOUR WALLET ADDRESS ====="
# Display wallet address (you'll need to enter your password)
echo "Enter your wallet password to display your Monero address:"
echo "address" | ~/monero/build/release/bin/monero-wallet-cli --wallet-file ~/monero-wallet/mywallet

echo "===== STEP 5: INSTALLING XMRIG MINER ====="
# Clone and build XMRig
cd ~
git clone https://github.com/xmrig/xmrig.git
cd xmrig
mkdir build
cd build
cmake ..
make -j$(nproc)

echo "===== STEP 6: CONFIGURING XMRIG FOR SUPPORTXMR POOL ====="
# Create a config file (replace YOUR_MONERO_WALLET_ADDRESS with your actual address)
cat > config.json << EOL
{
    "autosave": true,
    "cpu": true,
    "opencl": false,
    "cuda": false,
    "pools": [
        {
            "url": "pool.supportxmr.com:3333",
            "user": "YOUR_MONERO_WALLET_ADDRESS",
            "pass": "x",
            "keepalive": true,
            "tls": false,
            "rig-id": "AlpineCloudRig"
        }
    ]
}
EOL

echo "===== STEP 7: OPTIMIZING ALPINE FOR MINING ====="
# Disable unnecessary services to dedicate more resources to mining
rc-service -s

# Set CPU governor to performance if available
if [ -f /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
    echo "Setting CPU governor to performance mode..."
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo "performance" > $cpu 2>/dev/null
    done
fi

echo "===== SETUP COMPLETE ====="
echo ""
echo "IMPORTANT STEPS TO FINISH SETUP:"
echo "1. Replace 'YOUR_MONERO_WALLET_ADDRESS' in ~/xmrig/build/config.json with your actual address"
echo "2. To start mining: cd ~/xmrig/build && ./xmrig"
echo ""
echo "USEFUL COMMANDS:"
echo "- Check wallet balance: ~/monero/build/release/bin/monero-wallet-cli --wallet-file ~/monero-wallet/mywallet"
echo "- View mining stats: Visit https://supportxmr.com and enter your wallet address"
echo ""
echo "IMPORTANT NOTES:"
echo "- Keep your seed phrase safe - it's the only way to recover your wallet"
echo "- Monero takes time to sync with the network on first use"
echo "- Mining payouts usually require reaching a minimum threshold (0.1 XMR on SupportXMR)"
echo "- For better performance, consider using a lightweight version of the wallet instead of running a full node"
