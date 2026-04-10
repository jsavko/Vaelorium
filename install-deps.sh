#!/bin/bash
echo "Installing Tauri + Playwright system dependencies for WSL/Ubuntu..."
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  build-essential \
  libssl-dev
echo "Installing Playwright browser dependencies..."
npx playwright install-deps chromium
echo "Done!"
