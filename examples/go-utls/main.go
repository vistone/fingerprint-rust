package main

import (
	"bufio"
	"crypto/ecdh"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"strings"

	tls "github.com/refraction-networking/utls"
)

type Config struct {
	CipherSuites       []uint16          `json:"cipher_suites"`
	CompressionMethods []uint8           `json:"compression_methods"`
	Extensions         []ExtensionConfig `json:"extensions"`
	TLSVersMin         uint16            `json:"tls_vers_min"`
	TLSVersMax         uint16            `json:"tls_vers_max"`
}

type ExtensionConfig struct {
	Type string          `json:"type"`
	Data json.RawMessage `json:"data"`
}

type KeyShare struct {
	Group   uint16 `json:"group"`
	DataHex string `json:"data_hex"`
}

type PaddingConfig struct {
	PaddingLen int  `json:"padding_len"`
	WillPad    bool `json:"will_pad"`
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go <config.json> [url]")
		return
	}

	configFile := os.Args[1]
	targetUrl := "https://www.google.com"
	if len(os.Args) > 2 {
		targetUrl = os.Args[2]
	}

	fmt.Printf("Reading config from %s\n", configFile)
	jsonBytes, err := ioutil.ReadFile(configFile)
	if err != nil {
		panic(err)
	}

	var config Config
	err = json.Unmarshal(jsonBytes, &config)
	if err != nil {
		panic(err)
	}

	spec := &tls.ClientHelloSpec{
		CipherSuites:       config.CipherSuites,
		CompressionMethods: config.CompressionMethods,
		TLSVersMin:         config.TLSVersMin,
		TLSVersMax:         config.TLSVersMax,
		Extensions:         make([]tls.TLSExtension, 0),
		GetSessionID:       nil,
	}

	for _, extCfg := range config.Extensions {
		var ext tls.TLSExtension

		switch extCfg.Type {
		case "SNI":
			ext = &tls.SNIExtension{}
		case "StatusRequest":
			ext = &tls.StatusRequestExtension{}
		case "SupportedCurves":
			var curves []tls.CurveID
			json.Unmarshal(extCfg.Data, &curves)
            // Filter out curves we can't generate keys for (to avoid HRR failure)
            var filteredCurves []tls.CurveID
            for _, c := range curves {
                // Keep GREASE
                if (uint16(c) & 0x0f0f) == 0x0a0a {
                    filteredCurves = append(filteredCurves, c)
                    continue
                }
                // Keep standard curves
                if c == tls.X25519 || c == tls.CurveP256 || c == tls.CurveP384 {
                    filteredCurves = append(filteredCurves, c)
                } else {
                    // fmt.Printf("Dropping unsupported curve from SupportedGroups: %d\n", c)
                }
            }
			ext = &tls.SupportedCurvesExtension{Curves: filteredCurves}
		case "SupportedPoints":
			var points []uint8
			json.Unmarshal(extCfg.Data, &points)
			ext = &tls.SupportedPointsExtension{SupportedPoints: points}
		case "SignatureAlgorithms":
			var algs []tls.SignatureScheme
			json.Unmarshal(extCfg.Data, &algs)
			ext = &tls.SignatureAlgorithmsExtension{SupportedSignatureAlgorithms: algs}
		case "ALPN":
			// Force HTTP/1.1 to avoid h2 complexity for now
			ext = &tls.ALPNExtension{AlpnProtocols: []string{"http/1.1"}}
		case "ExtendedMasterSecret":
			ext = &tls.ExtendedMasterSecretExtension{}
		case "SessionTicket":
			ext = &tls.SessionTicketExtension{}
		case "SupportedVersions":
			var versions []uint16
			json.Unmarshal(extCfg.Data, &versions)
			ext = &tls.SupportedVersionsExtension{Versions: versions}
		case "PSKKeyExchangeModes":
            // RFC 8446: MUST be sent if and only if "pre_shared_key" is sent.
            // Since we don't support session tickets/PSK yet, we must skip this.
            continue
			/*
			var modes []uint8
			json.Unmarshal(extCfg.Data, &modes)
			ext = &tls.PSKKeyExchangeModesExtension{Modes: modes}
            */
		case "KeyShare":
			var shares []KeyShare
			json.Unmarshal(extCfg.Data, &shares)
			var keyShares []tls.KeyShare
			for _, s := range shares {
				curveID := tls.CurveID(s.Group)
				var data []byte

				// Check if GREASE: (val & 0x0f0f) == 0x0a0a
				if (s.Group & 0x0f0f) == 0x0a0a {
					data, _ = hex.DecodeString(s.DataHex)
				} else {
					// Generate Key Share
					var pubKey []byte
					
					switch curveID {
					case tls.X25519:
						curve := ecdh.X25519()
						priv, err := curve.GenerateKey(rand.Reader)
						if err == nil {
							pubKey = priv.PublicKey().Bytes()
						}
					case tls.CurveP256:
						curve := ecdh.P256()
						priv, err := curve.GenerateKey(rand.Reader)
						if err == nil {
							pubKey = priv.PublicKey().Bytes()
						}
					case tls.CurveP384:
						curve := ecdh.P384()
						priv, err := curve.GenerateKey(rand.Reader)
						if err == nil {
							pubKey = priv.PublicKey().Bytes()
						}
					default:
						// ML-KEM or others
						data, _ = hex.DecodeString(s.DataHex)
                        if len(data) == 0 {
                            // Don't send empty KeyShare
                            continue
                        }
					}
					
					if pubKey != nil {
						data = pubKey
					}
				}
				
				keyShares = append(keyShares, tls.KeyShare{Group: curveID, Data: data})
			}
			ext = &tls.KeyShareExtension{KeyShares: keyShares}

		case "SCT":
			ext = &tls.SCTExtension{}
		case "RenegotiationInfo":
			var renegotiation uint8
			json.Unmarshal(extCfg.Data, &renegotiation)
			ext = &tls.RenegotiationInfoExtension{Renegotiation: tls.RenegotiationSupport(renegotiation)}
		case "ApplicationSettings":
			var protocols []string
			json.Unmarshal(extCfg.Data, &protocols)
			ext = &tls.ApplicationSettingsExtension{SupportedProtocols: protocols}
		case "CompressCertificate":
			var algs []tls.CertCompressionAlgo
			json.Unmarshal(extCfg.Data, &algs)
			ext = &tls.UtlsCompressCertExtension{Algorithms: algs}
		case "GREASE":
			var val uint16
			json.Unmarshal(extCfg.Data, &val)
			ext = &tls.UtlsGREASEExtension{Value: val, Body: nil}
		case "Padding":
			var pad PaddingConfig
			json.Unmarshal(extCfg.Data, &pad)
			ext = &tls.UtlsPaddingExtension{GetPaddingLen: tls.BoringPaddingStyle}
		case "ECH":
			// Skip ECH to avoid empty payload issues
			continue
		}

		if ext != nil {
			spec.Extensions = append(spec.Extensions, ext)
		}
	}

	host := targetUrl
	host = strings.TrimPrefix(host, "https://")
	host = strings.TrimPrefix(host, "http://")
	path := "/"
	if idx := strings.Index(host, "/"); idx != -1 {
		path = host[idx:]
		host = host[:idx]
	}
	if !strings.Contains(host, ":") {
		host += ":443"
	}
	
	serverName, _, _ := net.SplitHostPort(host)

	fmt.Printf("Connecting to %s (SNI: %s)...\n", host, serverName)

	configTLS := &tls.Config{
		ServerName: serverName,
		InsecureSkipVerify: true,
	}

	dialer := net.Dialer{}
	conn, err := dialer.Dial("tcp", host)
	if err != nil {
		panic(fmt.Sprintf("Connect failed: %v", err))
	}

	uConn := tls.UClient(conn, configTLS, tls.HelloCustom)
	
	err = uConn.ApplyPreset(spec)
	if err != nil {
		panic(fmt.Sprintf("ApplyPreset failed: %v", err))
	}

	err = uConn.Handshake()
	if err != nil {
		panic(fmt.Sprintf("Handshake failed: %v", err))
	}

	state := uConn.ConnectionState()
	fmt.Printf("✅ Handshake successful! Protocol: %s\n", state.NegotiatedProtocol)
	
	req := fmt.Sprintf("GET %s HTTP/1.1\r\nHost: %s\r\nUser-Agent: Mozilla/5.0\r\nConnection: close\r\n\r\n", path, serverName)
	uConn.Write([]byte(req))
	
	resp, err := http.ReadResponse(bufio.NewReader(uConn), nil)
	if err != nil {
		fmt.Printf("Read response failed: %v\n", err)
	} else {
		fmt.Printf("Response Status: %s\n", resp.Status)
		body, _ := ioutil.ReadAll(resp.Body)
		if len(body) > 500 {
			fmt.Printf("Body (first 500 bytes):\n%s\n...", string(body[:500]))
		} else {
			fmt.Printf("Body:\n%s\n", string(body))
		}
	}
}
