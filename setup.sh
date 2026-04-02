#!/usr/bin/env bash
set -e

echo "Firescraper Setup"
echo "================="

# Check for yt-dlp
if command -v yt-dlp &> /dev/null; then
    echo "yt-dlp found: $(yt-dlp --version)"
else
    echo "yt-dlp not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install yt-dlp
    elif command -v pip3 &> /dev/null; then
        pip3 install yt-dlp
    elif command -v pip &> /dev/null; then
        pip install yt-dlp
    else
        echo "Error: Could not install yt-dlp. Install manually: https://github.com/yt-dlp/yt-dlp#installation"
        exit 1
    fi
    echo "yt-dlp installed: $(yt-dlp --version)"
fi

# Check for ffmpeg (needed by yt-dlp for audio conversion)
if command -v ffmpeg &> /dev/null; then
    echo "ffmpeg found"
else
    echo "ffmpeg not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install ffmpeg
    else
        echo "Error: Could not install ffmpeg. Install manually: https://ffmpeg.org/download.html"
        exit 1
    fi
    echo "ffmpeg installed"
fi

echo ""
echo "Setup complete! You can now scrape YouTube videos."
