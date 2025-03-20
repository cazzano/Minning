#!/bin/bash
# Complete Monero Setup Guide for Arch Linux

echo "===== STEP 1: INSTALLING MONERO WALLET ====="
# Install Monero wallet
sudo pacman -Syu --noconfirm
sudo pacman -S --noconfirm monero

echo "===== STEP 2: CREATING A MONERO WALLET ====="
# Create directory for wallet
mkdir -p ~/monero-wallet
cd ~/monero-wallet

# Generate new wallet (CLI method)
# This will prompt you to create a new wallet name and password
echo "Creating a new Monero wallet. Follow the prompts to set a wallet name and password."
echo "IMPORTANT: Write down your seed phrase! It's your only backup."
monero-wallet-cli --generate-new-wallet ~/monero-wallet/mywallet

# ALTERNATIVE: If you prefer the GUI wallet, comment out the above command and use:
# monero-wallet-gui

echo "===== STEP 3: GETTING YOUR WALLET ADDRESS ====="
# Display wallet address (you'll need to enter your password)
echo "Enter your wallet password to display your Monero address:"
echo "address" | monero-wallet-cli --wallet-file ~/monero-wallet/mywallet

echo "===== STEP 4: INSTALLING XMRIG MINER ====="
# Install dependencies for XMRig
sudo pacman -S --noconfirm base-devel git libmicrohttpd hwloc openssl cmake

# Clone and build XMRig
cd ~
git clone https://github.com/xmrig/xmrig.git
cd xmrig
mkdir build
cd build
cmake ..
make -j$(nproc)

echo "===== STEP 5: CONFIGURING XMRIG FOR SUPPORTXMR POOL ====="
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
            "rig-id": "ArchLinuxRig"
        }
    ]
}
EOL

echo "===== SETUP COMPLETE ====="
echo ""
echo "IMPORTANT STEPS TO FINISH SETUP:"
echo "1. Replace 'YOUR_MONERO_WALLET_ADDRESS' in ~/xmrig/build/config.json with your actual address"
echo "2. To start mining: cd ~/xmrig/build && ./xmrig"
echo ""
echo "USEFUL COMMANDS:"
echo "- Check wallet balance: monero-wallet-cli --wallet-file ~/monero-wallet/mywallet"
echo "- View mining stats: Visit https://supportxmr.com and enter your wallet address"
echo ""
echo "IMPORTANT NOTES:"
echo "- Keep your seed phrase safe - it's the only way to recover your wallet"
echo "- Monero takes time to sync with the network on first use"
echo "- Mining payouts usually require reaching a minimum threshold (0.1 XMR on SupportXMR)"
