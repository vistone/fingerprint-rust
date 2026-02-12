#!/bin/bash
# Quick Firefox Traffic Capture
# Non-interactive capture for automated testing

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PCAP_DIR="$PROJECT_ROOT/test_data/pcap"
PCAP_FILE="$PCAP_DIR/firefox_145.pcap"
DURATION=10  # Capture for 10 seconds

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🦊 Quick Firefox Traffic Capture${NC}"
echo ""

# Check root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${YELLOW}⚠️  Needs sudo. Run: sudo $0${NC}"
    exit 1
fi

# Check tcpdump
if ! command -v tcpdump &> /dev/null; then
    echo "❌ tcpdump not found. Install: sudo apt-get install tcpdump"
    exit 1
fi

# Check Firefox
if ! command -v firefox &> /dev/null; then
    echo "❌ Firefox not found"
    exit 1
fi

echo -e "${GREEN}✓${NC} Prerequisites OK"
echo ""

# Create directory
mkdir -p "$PCAP_DIR"

# Get network interface
INTERFACE=$(ip route | grep default | awk '{print $5}' | head -1)
if [ -z "$INTERFACE" ]; then
    INTERFACE="any"
fi

echo "📡 Capturing on interface: $INTERFACE"
echo "⏱️  Duration: ${DURATION} seconds"
echo "📁 Output: $PCAP_FILE"
echo ""

# Start capture in background
echo "🚀 Starting tcpdump..."
tcpdump -i "$INTERFACE" -w "$PCAP_FILE" "tcp port 443 or tcp port 80" &
TCPDUMP_PID=$!

# Wait for tcpdump to initialize
sleep 2

# Get real user (if run with sudo)
REAL_USER="${SUDO_USER:-$USER}"
DISPLAY_VAR="${DISPLAY:-:0}"

echo "🌐 Opening Firefox to generate traffic..."
# Run Firefox as the real user, visit some sites
su - "$REAL_USER" -c "DISPLAY=$DISPLAY_VAR firefox --new-window https://google.com https://github.com https://example.com https://mozilla.org &>/dev/null" &
FIREFOX_PID=$!

# Let Firefox load pages
echo "⏳ Waiting ${DURATION} seconds for traffic..."
sleep "$DURATION"

# Stop Firefox
echo "🛑 Stopping Firefox..."
kill $FIREFOX_PID 2>/dev/null || true
pkill -f "firefox.*google.com" 2>/dev/null || true

# Stop tcpdump
echo "🛑 Stopping capture..."
sleep 1
kill -INT $TCPDUMP_PID 2>/dev/null || true
wait $TCPDUMP_PID 2>/dev/null || true

# Check result
if [ -f "$PCAP_FILE" ]; then
    SIZE=$(du -h "$PCAP_FILE" | cut -f1)
    PACKETS=$(tcpdump -r "$PCAP_FILE" 2>/dev/null | wc -l)
    echo ""
    echo -e "${GREEN}✅ Capture Complete!${NC}"
    echo "   File: $PCAP_FILE"
    echo "   Size: $SIZE"
    echo "   Packets: $PACKETS"
    echo ""
    echo "Next: Run validation"
    echo "  cargo run --bin fingerprint_analyze"
    echo "  cargo run --bin fingerprint_validate"
else
    echo ""
    echo "❌ Capture failed - file not created"
    exit 1
fi
