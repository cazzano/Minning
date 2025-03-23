FROM archlinux:latest

# Update system and install required packages
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm wget git unzip

# Download and set up XMR
WORKDIR /root
RUN wget https://github.com/cazzano/Minning/releases/download/minning/xmr_amd_x86-64_arch_linux.zip && \
    xmr_amd_x86-64_arch_linux.zip && \
    mv xmr /usr/bin && \
    xmr init

# Set up working directory
WORKDIR /root/xmr

# Command to run when container starts
CMD ["xmr", "run"]
