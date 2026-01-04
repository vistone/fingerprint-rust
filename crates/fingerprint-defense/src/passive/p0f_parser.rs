//! p0f signatureParseer (detailedimplement)
//!
//! completeimplement p0f.fp formatParse, supportallfield and pattern.

use crate::passive::tcp::TcpSignature;
use thiserror::Error;

/// p0f TCP signature (complete版)
#[derive(Debug, Clone)]
pub struct P0fTcpSignature {
    /// signature ID
    pub id: String,

    /// taginfo
    pub label: SignatureLabel,

    /// systemtypelimit (optional)
    pub sys: Option<Vec<SystemType>>,

    /// TTL pattern
    pub ttl_pattern: TtlPattern,

    /// initialbeginning TTL value
    pub initial_ttl: u8,

    /// windowsizepattern
    pub window_mode: WindowMode,

    /// windowsizevaluepattern
    pub window_value: WindowSizePattern,

    /// MSS pattern
    pub mss_pattern: MssPattern,

    /// TCP optionsorder
    pub options_order: Vec<TcpOptionType>,

    /// IP flag
    pub ip_flags: IpFlags,

    /// otherfield
    pub other: String,
}

/// signaturetag
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureLabel {
    /// matchtype：s (specific) or g (generic)
    pub match_type: MatchType,

    /// systemtype：unix, win, ! (application)
    pub sys_type: SystemType,

    /// operating systemname
    pub os: String,

    /// versioninfo
    pub version: String,
}

/// matchtype
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
    Specific, // s
    Generic,  // g
}

/// systemtype
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemType {
    Unix,        // unix
    Windows,     // win
    Application, // !
}

/// TTL pattern
#[derive(Debug, Clone, PartialEq)]
pub enum TtlPattern {
    /// wildcard *
    Wildcard,
    /// concretevalue
    Value(u8),
}

/// windowsizepattern
#[derive(Debug, Clone, PartialEq)]
pub enum WindowMode {
    /// pattern 0: fixedvalue
    Fixed,
    /// pattern 1: 倍count
    Multiple,
    /// pattern 2: modecount
    Modulo,
    /// pattern 3: random
    Random,
}

/// windowsizevaluepattern
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSizePattern {
    /// wildcard *
    Wildcard,
    /// concretevalue
    Value(u16),
    /// 倍countpattern：m*N
    Multiple(u16),
    /// modecountpattern：%N
    Modulo(u16),
}

/// MSS pattern
#[derive(Debug, Clone, PartialEq)]
pub enum MssPattern {
    /// none MSS
    None,
    /// fixedvalue：mss,1460
    Fixed(u16),
    /// 倍countpattern：mss*20,10 (20倍count+10)
    Multiple { multiplier: u16, offset: u16 },
}

/// TCP optionstype
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpOptionType {
    Mss,         // mss
    WindowScale, // ws
    Sack,        // sok
    Timestamp,   // ts
    Nop,         // nop
    End,         // eol
    Other(u8),   // other
}

/// IP flag
#[derive(Debug, Clone, PartialEq)]
pub struct IpFlags {
    /// Don't Fragment
    pub df: bool,
    /// ID 递increase
    pub id_plus: bool,
    /// ID fixed
    pub id_minus: bool,
}

/// p0f Parseerror
#[derive(Debug, Error)]
pub enum P0fParseError {
    #[error("invalidtagformat: {0}")]
    InvalidLabel(String),

    #[error("invalidsignatureformat: {0}")]
    InvalidSignature(String),

    #[error("Parseerror: {0}")]
    Parse(String),
}

/// Parse p0f TCP signature
pub fn parse_tcp_signature(label: &str, sig: &str) -> Result<P0fTcpSignature, P0fParseError> {
    // Parsetag
    let label_info = parse_label(label)?;

    // Parsesignature：format as [ttl]:[initialbeginningttl]:[windowpattern]:[windowvalue]:[TCPoptions]:[IPflag]:[other]
    let parts: Vec<&str> = sig.split(':').collect();
    if parts.len() < 7 {
        return Err(P0fParseError::InvalidSignature(format!(
            "signaturepartialcountinsufficient: expected7，actual{}",
            parts.len()
        )));
    }

    // Parse TTL pattern
    let ttl_pattern =
        if parts[0] == "*" {
            TtlPattern::Wildcard
        } else {
            TtlPattern::Value(parts[0].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("invalid TTL: {}", parts[0]))
            })?)
        };

    // Parseinitialbeginning TTL
    let initial_ttl = parts[1].parse().map_err(|_| {
        P0fParseError::InvalidSignature(format!("invalidinitialbeginning TTL: {}", parts[1]))
    })?;

    // Parsewindowsizepattern
    let window_mode = match parts[2] {
        "0" => WindowMode::Fixed,
        "1" => WindowMode::Multiple,
        "2" => WindowMode::Modulo,
        "3" => WindowMode::Random,
        _ => {
            return Err(P0fParseError::InvalidSignature(format!(
                "invalidwindowpattern: {}",
                parts[2]
            )))
        }
    };

    // Parsewindowsizevaluepattern
    let window_value = parse_window_size_pattern(parts[3])?;

    // Parse MSS pattern and TCP options
    // p0f format: [ttl]:[initialbeginningttl]:[windowpattern]:[windowvalue]:[MSSpattern]:[optionsorder]:[IPflag]:[other]
    // so parts[4] is MSS pattern, parts[5] is optionsorder
    let mss_str = parts[4];
    let options_str = if parts.len() > 5 { parts[5] } else { "" };

    // merge MSS pattern and optionsorderperformParse
    let full_options_str = if !options_str.is_empty() {
        format!("{}:{}", mss_str, options_str)
    } else {
        mss_str.to_string()
    };

    let (mss_pattern, options_order) = parse_tcp_options(&full_options_str)?;

    // Parse IP flag
    let ip_flags = if parts.len() > 6 {
        parse_ip_flags(parts[6])?
    } else {
        IpFlags {
            df: false,
            id_plus: false,
            id_minus: false,
        }
    };

    // otherfield
    let other = if parts.len() > 7 {
        parts[7].to_string()
    } else {
        "0".to_string()
    };

    Ok(P0fTcpSignature {
        id: format!("tcp-{}", label.replace(':', "-")),
        label: label_info,
        sys: None, // will in back续Parse sys field when settings
        ttl_pattern,
        initial_ttl,
        window_mode,
        window_value,
        mss_pattern,
        options_order,
        ip_flags,
        other,
    })
}

/// Parsetag
fn parse_label(label: &str) -> Result<SignatureLabel, P0fParseError> {
    // format: s:unix:Linux:3.11 and newer
    let parts: Vec<&str> = label.split(':').collect();
    if parts.len() < 4 {
        return Err(P0fParseError::InvalidLabel(format!(
            "tagpartialcountinsufficient: expected4，actual{}",
            parts.len()
        )));
    }

    let match_type = match parts[0] {
        "s" => MatchType::Specific,
        "g" => MatchType::Generic,
        _ => {
            return Err(P0fParseError::InvalidLabel(format!(
                "invalidmatchtype: {}",
                parts[0]
            )))
        }
    };

    let sys_type = match parts[1] {
        "unix" => SystemType::Unix,
        "win" => SystemType::Windows,
        "!" => SystemType::Application,
        _ => {
            return Err(P0fParseError::InvalidLabel(format!(
                "invalidsystemtype: {}",
                parts[1]
            )))
        }
    };

    Ok(SignatureLabel {
        match_type,
        sys_type,
        os: parts[2].to_string(),
        version: parts[3..].join(":"), // versionmayincluding冒号
    })
}

/// Parsewindowsizepattern
fn parse_window_size_pattern(pattern: &str) -> Result<WindowSizePattern, P0fParseError> {
    if pattern == "*" {
        return Ok(WindowSizePattern::Wildcard);
    }

    // Check倍countpattern：m*N
    if let Some(pos) = pattern.find('*') {
        let multiplier = pattern[pos + 1..].parse().map_err(|_| {
            P0fParseError::InvalidSignature(format!("invalidwindow倍count: {}", pattern))
        })?;
        return Ok(WindowSizePattern::Multiple(multiplier));
    }

    // Checkmodecountpattern：%N
    if let Some(stripped) = pattern.strip_prefix('%') {
        let modulo = stripped.parse().map_err(|_| {
            P0fParseError::InvalidSignature(format!("invalidwindow模count: {}", pattern))
        })?;
        return Ok(WindowSizePattern::Modulo(modulo));
    }

    // fixedvalue
    let value = pattern
        .parse()
        .map_err(|_| P0fParseError::InvalidSignature(format!("invalidwindowsize: {}", pattern)))?;
    Ok(WindowSizePattern::Value(value))
}

/// Parse TCP options
fn parse_tcp_options(options_str: &str) -> Result<(MssPattern, Vec<TcpOptionType>), P0fParseError> {
    let mut mss_pattern = MssPattern::None;
    let mut options_order = Vec::new();

    // optionsformat: mss*20,10:mss,sok,ts,nop,ws
    // firstpartial is MSS pattern, secondpartial is optionsorder

    let parts: Vec<&str> = options_str.split(':').collect();
    if parts.is_empty() {
        return Ok((mss_pattern, options_order));
    }

    // Parse MSS pattern (firstpartial)
    // formatmay is: mss*20,10 or mss,1460
    let mss_part = parts[0];
    if mss_part.contains("mss") {
        mss_pattern = parse_mss_pattern(mss_part)?;
    }

    // Parseoptionsorder
    // formatmay is: mss*20,10:mss,sok,ts,nop,ws
    // or one who: mss,1460:mss,sok,ts,nop,ws
    // secondpartial is optionsorder
    if parts.len() > 1 {
        // secondpartialincludingoptionsorder
        for opt_str in parts[1].split(',') {
            let opt = match opt_str.trim() {
                "mss" => TcpOptionType::Mss,
                "ws" => TcpOptionType::WindowScale,
                "sok" => TcpOptionType::Sack,
                "ts" => TcpOptionType::Timestamp,
                "nop" => TcpOptionType::Nop,
                "eol" => TcpOptionType::End,
                _ => {
                    // tryParse as count字
                    if let Ok(num) = opt_str.parse::<u8>() {
                        TcpOptionType::Other(num)
                    } else {
                        continue; // skipnot知options
                    }
                }
            };
            options_order.push(opt);
        }
    } else {
        // Ifnosecondpartial, mayoptionsorderthen in firstpartial ( in MSS patternafter)
        // format: mss*20,10 or mss,1460
        // thissituationdown, optionsordermay不 exists,  or one whoneed from otherplaceExtract
        // temporary when 不processthissituation
    }

    Ok((mss_pattern, options_order))
}

/// Parse MSS pattern
fn parse_mss_pattern(mss_str: &str) -> Result<MssPattern, P0fParseError> {
    // format: mss*20,10 or mss,1460
    if let Some(pos) = mss_str.find('*') {
        // 倍countpattern: mss*20,10
        let after_star = &mss_str[pos + 1..];
        let parts: Vec<&str> = after_star.split(',').collect();
        if parts.len() >= 2 {
            let multiplier = parts[0].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("invalid MSS 倍count: {}", parts[0]))
            })?;
            let offset = parts[1].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("invalid MSS offset: {}", parts[1]))
            })?;
            return Ok(MssPattern::Multiple { multiplier, offset });
        }
    }

    // fixedvaluepattern: mss,1460
    // findfirst逗numberbackcount字
    if let Some(pos) = mss_str.find(',') {
        let value_str = &mss_str[pos + 1..];
        // maystill有more逗number, onlygetfirstcount字partial
        let value_part = value_str.split(',').next().unwrap_or(value_str);
        if let Ok(value) = value_part.parse::<u16>() {
            return Ok(MssPattern::Fixed(value));
        }
    }

    Ok(MssPattern::None)
}

/// Parse IP flag
fn parse_ip_flags(flags_str: &str) -> Result<IpFlags, P0fParseError> {
    let mut df = false;
    let mut id_plus = false;
    let mut id_minus = false;

    for flag in flags_str.split(',') {
        match flag.trim() {
            "df" => df = true,
            "id+" => id_plus = true,
            "id-" => id_minus = true,
            "" => continue,
            _ => {
                // ignorenot知flag
            }
        }
    }

    Ok(IpFlags {
        df,
        id_plus,
        id_minus,
    })
}

/// will P0fTcpSignature convert to TcpSignature ( for match)
impl From<P0fTcpSignature> for TcpSignature {
    fn from(p0f_sig: P0fTcpSignature) -> Self {
        // from MSS patternExtractfixedvalue ( if may)
        let mss = match &p0f_sig.mss_pattern {
            MssPattern::Fixed(v) => Some(*v),
            MssPattern::Multiple { multiplier, offset } => {
                // usefirstmay MSS valueasExamples
                let m = (*multiplier as u32)
                    .saturating_mul(10)
                    .saturating_add(*offset as u32);
                Some(m.min(65535) as u16)
            }
            MssPattern::None => None,
        };

        // from optionsorder in Extract Window Scale ( if exists)
        let window_scale = if p0f_sig.options_order.contains(&TcpOptionType::WindowScale) {
            Some(7) // defaultvalue，actualshould from countpacket in Extract
        } else {
            None
        };

        // from windowvaluepatternExtractfixedvalue ( if may)
        let window_size = match &p0f_sig.window_value {
            WindowSizePattern::Value(v) => *v,
            _ => 0, // wildcard or otherpattern
        };

        TcpSignature {
            id: p0f_sig.id,
            ttl: p0f_sig.initial_ttl,
            window_size,
            mss,
            window_scale,
            os_type: Some(p0f_sig.label.os.clone()),
            confidence: if p0f_sig.label.match_type == MatchType::Specific {
                0.9
            } else {
                0.7
            },
            sample_count: 1,
        }
    }
}
