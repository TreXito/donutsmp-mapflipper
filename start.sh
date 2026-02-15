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

# Check if config.json exists
if [ ! -f "config.json" ]; then
    echo "Warning: config.json not found"
    echo "Creating config.json from config.example.json..."
    if [ -f "config.example.json" ]; then
        cp config.example.json config.json
        echo "Please edit config.json and set your Minecraft username"
        echo ""
    else
        echo "Error: config.example.json not found"
        exit 1
    fi
fi

# Check if BOT_USERNAME is set (fallback if config.json isn't configured)
if [ -z "$BOT_USERNAME" ]; then
    echo "Note: BOT_USERNAME environment variable is not set"
    echo "Make sure your username is configured in config.json"
    echo ""
fi

echo "Starting bot..."
echo ""
node bot.js
