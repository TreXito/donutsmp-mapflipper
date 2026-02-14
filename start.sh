#!/bin/bash

# DonutSMP Map Flipper Bot Startup Script

echo "=================================="
echo "DonutSMP Map Flipper Bot"
echo "=================================="
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed"
    echo "Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
    echo ""
fi

# Check if BOT_USERNAME is set
if [ -z "$BOT_USERNAME" ]; then
    echo "Warning: BOT_USERNAME environment variable is not set"
    echo "Please set it or edit the CONFIG in bot.js"
    echo ""
    echo "Usage: BOT_USERNAME=YourUsername ./start.sh"
    echo ""
fi

echo "Starting bot..."
echo ""
node bot.js
