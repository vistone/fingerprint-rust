#!/bin/bash
# Smart Traffic Capture Wizard with Real-time Guidance
# Automated browser traffic capture for fingerprinting validation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PCAP_DIR="$PROJECT_ROOT/test_data/pcap"
EXPECTED_DIR="$PROJECT_ROOT/test_data/expected"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Unicode symbols
CHECK="✓"
CROSS="✗"
ARROW="→"
STAR="★"

clear

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Smart Browser Traffic Capture Wizard                     ║${NC}"
echo -e "${CYAN}║  Phase 2: Real-World Fingerprint Validation              ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check prerequisites
echo -e "${BLUE}${ARROW} Checking prerequisites...${NC}"

# Check root privileges
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}${CROSS} This script requires root privileges${NC}"
    echo -e "${YELLOW}Please run: sudo $0${NC}"
    exit 1
fi
echo -e "${GREEN}${CHECK} Root privileges confirmed${NC}"

# Check tcpdump
if ! command -v tcpdump &> /dev/null; then
    echo -e "${RED}${CROSS} tcpdump not found${NC}"
    echo -e "${YELLOW}Install with: sudo apt-get install tcpdump${NC}"
    exit 1
fi
echo -e "${GREEN}${CHECK} tcpdump available${NC}"

# Create directories
mkdir -p "$PCAP_DIR" "$EXPECTED_DIR"
echo -e "${GREEN}${CHECK} Directories ready${NC}"
echo ""

# Display capture plan
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}${STAR} Capture Plan:${NC}"
echo ""
echo -e "  We'll capture traffic from 3 major browsers:"
echo -e "  ${GREEN}1.${NC} Chrome (latest)"
echo -e "  ${GREEN}2.${NC} Firefox (latest)"
echo -e "  ${GREEN}3.${NC} Safari (macOS only)"
echo ""
echo -e "  ${BLUE}Test websites:${NC}"
echo -e "    • https://google.com (reliable, fast)"
echo -e "    • https://github.com (HTTPS, modern)"
echo -e "    • https://example.com (simple test)"
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Function to capture traffic
capture_browser_traffic() {
    local browser_name=$1
    local version=$2
    local duration=${3:-30}
    
    local output_file="$PCAP_DIR/${browser_name}_${version}.pcap"
    
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║  Capturing: ${browser_name} v${version}${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    echo -e "${YELLOW}📋 Instructions:${NC}"
    echo -e "  ${GREEN}1.${NC} Open ${browser_name} browser"
    echo -e "  ${GREEN}2.${NC} Visit these sites IN ORDER:"
    echo -e "      ${ARROW} https://google.com"
    echo -e "      ${ARROW} https://github.com"
    echo -e "      ${ARROW} https://example.com"
    echo -e "  ${GREEN}3.${NC} Wait for pages to fully load"
    echo -e "  ${GREEN}4.${NC} ${RED}Do NOT close the browser${NC} during capture"
    echo ""
    echo -e "${BLUE}⏱  Capture duration: ${duration} seconds${NC}"
    echo ""
    
    read -p "$(echo -e ${GREEN}Press ENTER when ${browser_name} is ready...${NC})" 
    
    echo -e "${YELLOW}🔴 Capture starting in 3 seconds...${NC}"
    sleep 1
    echo -e "${YELLOW}   2...${NC}"
    sleep 1
    echo -e "${YELLOW}   1...${NC}"
    sleep 1
    
    echo -e "${GREEN}${CHECK} CAPTURING NOW!${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Start capture in background with progress indicator
    tcpdump -i any -w "$output_file" "tcp port 443" 2>/dev/null &
    TCPDUMP_PID=$!
    
    # Animated progress bar
    for i in $(seq 1 $duration); do
        percentage=$((i * 100 / duration))
        filled=$((percentage / 2))
        empty=$((50 - filled))
        
        printf "\r${BLUE}["
        printf "%${filled}s" | tr ' ' '█'
        printf "%${empty}s" | tr ' ' '░'
        printf "] ${percentage}%% (${i}/${duration}s)${NC}"
        
        sleep 1
    done
    
    # Stop capture
    kill $TCPDUMP_PID 2>/dev/null || true
    wait $TCPDUMP_PID 2>/dev/null || true
    
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Verify capture
    if [ -f "$output_file" ]; then
        local size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null)
        local packet_count=$(tcpdump -r "$output_file" 2>/dev/null | wc -l)
        
        if [ $size -gt 1000 ] && [ $packet_count -gt 10 ]; then
            echo -e "${GREEN}${CHECK} Capture successful!${NC}"
            echo -e "   File: $output_file"
            echo -e "   Size: $(numfmt --to=iec-i --suffix=B $size 2>/dev/null || echo ${size}B)"
            echo -e "   Packets: $packet_count"
            
            # Create expected result template
            create_expected_result "$browser_name" "$version"
            
            return 0
        else
            echo -e "${YELLOW}⚠  Capture succeeded but file seems small${NC}"
            echo -e "   Size: $size bytes, Packets: $packet_count"
            echo -e "   This might be okay if you visited sites quickly"
            return 1
        fi
    else
        echo -e "${RED}${CROSS} Capture failed - file not created${NC}"
        return 1
    fi
}

# Function to create expected result
create_expected_result() {
    local browser=$1
    local version=$2
    local expected_file="$EXPECTED_DIR/${browser}_${version}.json"
    
    # Extract major version
    local version_major=$(echo $version | cut -d'.' -f1)
    
    cat > "$expected_file" << EOF
{
  "browser": "$browser",
  "version": "$version",
  "version_major": $version_major,
  "os": "$(uname -s)",
  "confidence_min": 0.90,
  "layers": {
    "tcp": {
      "detected": true,
      "confidence_min": 0.75
    },
    "tls": {
      "detected": true,
      "confidence_min": 0.85
    },
    "http2": {
      "detected": true,
      "confidence_min": 0.90
    }
  },
  "captured_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "test_sites": [
    "https://google.com",
    "https://github.com",
    "https://example.com"
  ]
}
EOF
    
    echo -e "${GREEN}${CHECK} Expected result template created: $expected_file${NC}"
}

# Interactive browser selection
echo -e "${YELLOW}Select browsers to capture (space-separated numbers):${NC}"
echo -e "  ${GREEN}1.${NC} Chrome"
echo -e "  ${GREEN}2.${NC} Firefox"
echo -e "  ${GREEN}3.${NC} Safari (macOS only)"
echo -e "  ${GREEN}4.${NC} All available"
echo ""
read -p "$(echo -e ${CYAN}Your choice: ${NC})" choices

# Detect browser versions
detect_version() {
    local browser=$1
    local version=""
    
    case $browser in
        "chrome")
            if command -v google-chrome &> /dev/null; then
                version=$(google-chrome --version 2>/dev/null | grep -oP '\d+\.\d+\.\d+\.\d+' | head -1)
            elif command -v chromium &> /dev/null; then
                version=$(chromium --version 2>/dev/null | grep -oP '\d+\.\d+\.\d+\.\d+' | head -1)
            fi
            ;;
        "firefox")
            if command -v firefox &> /dev/null; then
                version=$(firefox --version 2>/dev/null | grep -oP '\d+\.\d+' | head -1)
            fi
            ;;
        "safari")
            if [[ "$OSTYPE" == "darwin"* ]]; then
                version=$(defaults read /Applications/Safari.app/Contents/Info CFBundleShortVersionString 2>/dev/null || echo "17.0")
            fi
            ;;
    esac
    
    echo "$version"
}

# Process captures
captured=0
failed=0

if [[ "$choices" == *"1"* ]] || [[ "$choices" == *"4"* ]]; then
    version=$(detect_version "chrome")
    if [ -z "$version" ]; then
        echo -e "${YELLOW}⚠  Chrome version not detected, using default: 136${NC}"
        version="136"
    fi
    
    if capture_browser_traffic "Chrome" "$version" 30; then
        ((captured++))
    else
        ((failed++))
    fi
fi

if [[ "$choices" == *"2"* ]] || [[ "$choices" == *"4"* ]]; then
    version=$(detect_version "firefox")
    if [ -z "$version" ]; then
        echo -e "${YELLOW}⚠  Firefox version not detected, using default: 135${NC}"
        version="135"
    fi
    
    if capture_browser_traffic "Firefox" "$version" 30; then
        ((captured++))
    else
        ((failed++))
    fi
fi

if [[ "$choices" == *"3"* ]] || [[ "$choices" == *"4"* ]]; then
    if [[ "$OSTYPE" != "darwin"* ]]; then
        echo -e "${RED}${CROSS} Safari is only available on macOS${NC}"
        ((failed++))
    else
        version=$(detect_version "safari")
        if [ -z "$version" ]; then
            version="17.0"
        fi
        
        if capture_browser_traffic "Safari" "$version" 30; then
            ((captured++))
        else
            ((failed++))
        fi
    fi
fi

# Summary
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Capture Summary                                           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${GREEN}Successful:${NC} $captured"
echo -e "  ${RED}Failed:${NC}     $failed"
echo ""
echo -e "${YELLOW}📁 PCAP files saved to:${NC}"
ls -lh "$PCAP_DIR"/*.pcap 2>/dev/null || echo "  No files"
echo ""

# Next steps
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Next Steps                                                ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${GREEN}1.${NC} Run validation tests:"
echo -e "     ${BLUE}cargo test --package fingerprint --test validation -- --ignored${NC}"
echo ""
echo -e "  ${GREEN}2.${NC} Analyze captured traffic:"
echo -e "     ${BLUE}cargo run --bin fingerprint-analyze${NC}"
echo ""
echo -e "  ${GREEN}3.${NC} Generate accuracy report:"
echo -e "     ${BLUE}cargo run --bin fingerprint-validate${NC}"
echo ""
echo -e "${GREEN}${CHECK} Phase 2 capture complete!${NC}"
