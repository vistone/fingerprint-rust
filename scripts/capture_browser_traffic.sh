#!/bin/bash
# Browser Traffic Capture Script
# Captures real browser traffic for fingerprinting validation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PCAP_DIR="$PROJECT_ROOT/test_data/pcap"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Browser Traffic Capture Tool${NC}"
echo "======================================"
echo ""

# Check if tcpdump is available
if ! command -v tcpdump &> /dev/null; then
    echo -e "${RED}Error: tcpdump is not installed${NC}"
    echo "Install it with: sudo apt-get install tcpdump (Debian/Ubuntu)"
    echo "                 sudo yum install tcpdump (RHEL/CentOS)"
    exit 1
fi

# Check for root privileges
if [ "$EUID" -ne 0 ]; then 
    echo -e "${YELLOW}Warning: This script requires root privileges${NC}"
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Create PCAP directory if it doesn't exist
mkdir -p "$PCAP_DIR"

# Function to capture traffic
capture_traffic() {
    local browser=$1
    local version=$2
    local duration=${3:-30}
    local port=${4:-443}
    local output_file="$PCAP_DIR/${browser}_${version}.pcap"
    
    echo -e "${GREEN}Capturing $browser $version traffic...${NC}"
    echo "  Duration: ${duration}s"
    echo "  Port: $port"
    echo "  Output: $output_file"
    echo ""
    echo -e "${YELLOW}Please open $browser and visit HTTPS websites (e.g., google.com, github.com)${NC}"
    echo "Press Enter when ready to start capture..."
    read
    
    echo "Capturing traffic for ${duration} seconds..."
    timeout $duration tcpdump -i any -w "$output_file" "tcp port $port" 2>/dev/null || true
    
    if [ -f "$output_file" ]; then
        local size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null)
        echo -e "${GREEN}✓ Capture complete: $output_file ($size bytes)${NC}"
        
        # Show packet count
        local packet_count=$(tcpdump -r "$output_file" 2>/dev/null | wc -l)
        echo "  Packets captured: $packet_count"
    else
        echo -e "${RED}✗ Capture failed${NC}"
    fi
    echo ""
}

# Main menu
echo "Select browser to capture:"
echo "1) Chrome (latest)"
echo "2) Firefox (latest)"
echo "3) Safari (macOS only)"
echo "4) Edge (Chromium-based)"
echo "5) Custom (manual input)"
echo "6) Capture all (automated sequence)"
echo "0) Exit"
echo ""
read -p "Choice: " choice

case $choice in
    1)
        read -p "Enter Chrome version (e.g., 136): " version
        version=${version:-136}
        capture_traffic "chrome" "$version" 30 443
        ;;
    2)
        read -p "Enter Firefox version (e.g., 135): " version
        version=${version:-135}
        capture_traffic "firefox" "$version" 30 443
        ;;
    3)
        if [[ "$OSTYPE" != "darwin"* ]]; then
            echo -e "${RED}Safari is only available on macOS${NC}"
            exit 1
        fi
        read -p "Enter Safari version (e.g., 17): " version
        version=${version:-17}
        capture_traffic "safari" "$version" 30 443
        ;;
    4)
        read -p "Enter Edge version (e.g., 136): " version
        version=${version:-136}
        capture_traffic "edge" "$version" 30 443
        ;;
    5)
        read -p "Enter browser name: " browser
        read -p "Enter version: " version
        read -p "Enter duration (seconds, default 30): " duration
        duration=${duration:-30}
        capture_traffic "$browser" "$version" "$duration" 443
        ;;
    6)
        echo -e "${YELLOW}Automated capture mode${NC}"
        echo "Will capture: Chrome, Firefox, Edge (30s each)"
        echo "Press Ctrl+C to cancel..."
        sleep 3
        
        capture_traffic "chrome" "136" 30 443
        capture_traffic "firefox" "135" 30 443
        capture_traffic "edge" "136" 30 443
        
        echo -e "${GREEN}All captures complete!${NC}"
        ;;
    0)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Capture Summary${NC}"
echo "======================================"
echo "Captured PCAP files:"
ls -lh "$PCAP_DIR"/*.pcap 2>/dev/null || echo "No PCAP files found"
echo ""
echo "Next steps:"
echo "1. Run integration tests: cargo test --test e2e_fingerprint"
echo "2. Analyze PCAP: cargo run --bin fingerprint -- analyze <pcap_file>"
echo "3. Generate report: cargo run --bin fingerprint -- validate"
