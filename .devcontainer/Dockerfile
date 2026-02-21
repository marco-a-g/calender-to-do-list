FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Installation der System-Abhängigkeiten für Dioxus/Desktop (WebKit 4.1 + Soup 3)
RUN apt-get update && apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libsoup-3.0-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libxdo-dev \
    pkg-config \
    xauth \
    x11-apps \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Dioxus-CLI installieren
USER vscode
RUN cargo install dioxus-cli