//! p0f signature parsed er (detailedimplement)
//!
//! completeimplement p0f.fp format parsed ，support all field and pattern。

use crate::passive::tcp::TcpSignature;
use thiserror::Error;

/// p0f TCP signature (complete version)
#[derive(Debug, Clone)]
pub struct P0fTcpSignature {
 /// signature ID
 pub id: String,

 /// taginfo
 pub label: SignatureLabel,

 /// system typelimit (optional)
 pub sys: Option<Vec<SystemType>>,

 /// TTL pattern
 pub ttl_pattern: TtlPattern,

 /// initialbeginning TTL value
 pub initial_ttl: u8,

 /// window sizepattern
 pub window _mode: WindowMode,

 /// window sizevaluepattern
 pub window _value: WindowSizePattern,

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

 /// system type：unix, win,! (application)
 pub sys_type: SystemType,

 /// operating system name
 pub os: String,

 /// versioninfo
 pub version: String,
}

/// matchtype
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
 Specific, // s
 Generic, // g
}

/// system type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemType {
 Unix, // unix
 Windows, // win
 Application, //!
}

/// TTL pattern
#[derive(Debug, Clone, PartialEq)]
pub enum TtlPattern {
 /// wildcard *
 Wildcard,
 /// concretevalue
 Value(u8),
}

/// window sizepattern
#[derive(Debug, Clone, PartialEq)]
pub enum WindowMode {
 /// pattern 0: fixedvalue
 Fixed,
 /// pattern 1: times count
 Multiple,
 /// pattern 2: 模count
 Modulo,
 /// pattern 3: random
 Random,
}

/// window sizevaluepattern
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSizePattern {
 /// wildcard *
 Wildcard,
 /// concretevalue
 Value(u16),
 /// times countpattern：m*N
 Multiple(u16),
 /// 模countpattern：%N
 Modulo(u16),
}

/// MSS pattern
#[derive(Debug, Clone, PartialEq)]
pub enum MssPattern {
 /// no MSS
 None,
 /// fixedvalue：mss,1460
 Fixed(u16),
 /// times countpattern：mss*20,10 (20 times count+10)
 Multiple { multiplier: u16, offset: u16 },
}

/// TCP optionstype
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpOptionType {
 Mss, // mss
 WindowScale, // ws
 Sack, // sok
 Timestamp, // ts
 Nop, // nop
 End, // eol
 Other(u8), // other
}

/// IP flag
#[derive(Debug, Clone, PartialEq)]
pub struct IpFlags {
 /// Don't Fragment
 pub df: bool,
 /// ID 递增
 pub id_plus: bool,
 /// ID fixed
 pub id_minus: bool,
}

/// p0f parsed error
#[derive(Debug, Error)]
pub enum P0f parsed Error {
 #[error("invalidtagformat: {0}")]
 InvalidLabel(String),

 #[error("invalidsignatureformat: {0}")]
 InvalidSignature(String),

 #[error(" parsed error: {0}")]
 parsed (String),
}

/// parsed p0f TCP signature
pub fn parse_tcp_signature(label: &str, sig: &str) -> Result<P0fTcpSignature, P0f parsed Error> {
 // parsed tag
 let label_info = parse_label(label)?;

 // parsed signature：format as [ttl]:[initialbeginningttl]:[ window pattern]:[ window value]:[TCPoptions]:[IPflag]:[other]
 let parts: Vec<&str> = sig.split(':').collect();
 if parts.len() < 7 {
 return Err(P0f parsed Error::InvalidSignature(format!(
 "signaturepartialcountinsufficient: expected7，actual{}",
 parts.len()
)));
 }

 // parsed TTL pattern
 let ttl_pattern =
 if parts[0] == "*" {
 TtlPattern::Wildcard
 } else {
 TtlPattern::Value(parts[0].parse().map_err(|_| {
 P0f parsed Error::InvalidSignature(format!("invalid TTL: {}", parts[0]))
 })?)
 };

 // parsed initialbeginning TTL
 let initial_ttl = parts[1]
.parse()
.map_err(|_| P0f parsed Error::InvalidSignature(format!("invalidinitialbeginning TTL: {}", parts[1])))?;

 // parsed window sizepattern
 let window _mode = match parts[2] {
 "0" => WindowMode::Fixed,
 "1" => WindowMode::Multiple,
 "2" => WindowMode::Modulo,
 "3" => WindowMode::Random,
 _ => {
 return Err(P0f parsed Error::InvalidSignature(format!(
 "invalid window pattern: {}",
 parts[2]
)))
 }
 };

 // parsed window sizevaluepattern
 let window _value = parse_ window _size_pattern(parts[3])?;

 // parsed MSS pattern and TCP options
 // p0f format: [ttl]:[initialbeginningttl]:[ window pattern]:[ window value]:[MSSpattern]:[optionsorder]:[IPflag]:[other]
 // so parts[4] is MSS pattern，parts[5] is optionsorder
 let mss_str = parts[4];
 let options_str = if parts.len() > 5 { parts[5] } else { "" };

 // merge MSS pattern and optionsorder perform parsed 
 let full_options_str = if!options_str.is_empty() {
 format!("{}:{}", mss_str, options_str)
 } else {
 mss_str.to_string()
 };

 let (mss_pattern, options_order) = parse_tcp_options(&full_options_str)?;

 // parsed IP flag
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
 sys: None, // will in back续 parsed sys field when settings
 ttl_pattern,
 initial_ttl,
 window _mode,
 window _value,
 mss_pattern,
 options_order,
 ip_flags,
 other,
 })
}

/// parsed tag
fn parse_label(label: &str) -> Result<SignatureLabel, P0f parsed Error> {
 // format: s:unix:Linux:3.11 and newer
 let parts: Vec<&str> = label.split(':').collect();
 if parts.len() < 4 {
 return Err(P0f parsed Error::InvalidLabel(format!(
 "tagpartialcountinsufficient: expected4，actual{}",
 parts.len()
)));
 }

 let match_type = match parts[0] {
 "s" => MatchType::Specific,
 "g" => MatchType::Generic,
 _ => {
 return Err(P0f parsed Error::InvalidLabel(format!(
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
 return Err(P0f parsed Error::InvalidLabel(format!(
 "invalidsystem type: {}",
 parts[1]
)))
 }
 };

 Ok(SignatureLabel {
 match_type,
 sys_type,
 os: parts[2].to_string(),
 version: parts[3..].join(":"), // versionmayincluding冒 number 
 })
}

/// parsed window sizepattern
fn parse_ window _size_pattern(pattern: &str) -> Result<WindowSizePattern, P0f parsed Error> {
 if pattern == "*" {
 return Ok(WindowSizePattern::Wildcard);
 }

 // Check times countpattern：m*N
 if let Some(pos) = pattern.find('*') {
 let multiplier = pattern[pos + 1..]
.parse()
.map_err(|_| P0f parsed Error::InvalidSignature(format!("invalid window times count: {}", pattern)))?;
 return Ok(WindowSizePattern::Multiple(multiplier));
 }

 // Check模countpattern：%N
 if let Some(stripped) = pattern.strip_prefix('%') {
 let modulo = stripped
.parse()
.map_err(|_| P0f parsed Error::InvalidSignature(format!("invalid window 模count: {}", pattern)))?;
 return Ok(WindowSizePattern::Modulo(modulo));
 }

 // fixedvalue
 let value = pattern
.parse()
.map_err(|_| P0f parsed Error::InvalidSignature(format!("invalid window size: {}", pattern)))?;
 Ok(WindowSizePattern::Value(value))
}

/// parsed TCP options
fn parse_tcp_options(options_str: &str) -> Result<(MssPattern, Vec<TcpOptionType>), P0f parsed Error> {
 let mut mss_pattern = MssPattern::None;
 let mut options_order = Vec::new();

 // optionsformat: mss*20,10:mss,sok,ts,nop,ws
 // firstpartial is MSS pattern，secondpartial is optionsorder

 let parts: Vec<&str> = options_str.split(':').collect();
 if parts.is_empty() {
 return Ok((mss_pattern, options_order));
 }

 // parsed MSS pattern (firstpartial)
 // formatmay is: mss*20,10 or mss,1460
 let mss_part = parts[0];
 if mss_part.contains("mss") {
 mss_pattern = parse_mss_pattern(mss_part)?;
 }

 // parsed optionsorder
 // formatmay is: mss*20,10:mss,sok,ts,nop,ws
 // or 者: mss,1460:mss,sok,ts,nop,ws
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
 // try parsed as count字
 if let Ok(num) = opt_str.parse::<u8>() {
 TcpOptionType::Other(num)
 } else {
 continue; // skipnot know options
 }
 }
 };
 options_order.push(opt);
 }
 } else {
 // Ifnosecondpartial, mayoptionsorder just in firstpartial (in MSS patternafter)
 // format: mss*20,10 or mss,1460
 // thissituationdown，optionsordermay not exists， or 者need from other地方Extract
 // temporary when not processthissituation
 }

 Ok((mss_pattern, options_order))
}

/// parsed MSS pattern
fn parse_mss_pattern(mss_str: &str) -> Result<MssPattern, P0f parsed Error> {
 // format: mss*20,10 or mss,1460
 if let Some(pos) = mss_str.find('*') {
 // times countpattern: mss*20,10
 let after_star = &mss_str[pos + 1..];
 let parts: Vec<&str> = after_star.split(',').collect();
 if parts.len() >= 2 {
 let multiplier = parts[0].parse().map_err(|_| {
 P0f parsed Error::InvalidSignature(format!("invalid MSS times count: {}", parts[0]))
 })?;
 let offset = parts[1].parse().map_err(|_| {
 P0f parsed Error::InvalidSignature(format!("invalid MSS offset: {}", parts[1]))
 })?;
 return Ok(MssPattern::Multiple { multiplier, offset });
 }
 }

 // fixedvaluepattern: mss,1460
 // findfirst逗 number backcount字
 if let Some(pos) = mss_str.find(',') {
 let value_str = &mss_str[pos + 1..];
 // maystill have more逗 number ， only 取firstcount字partial
 let value_part = value_str.split(',').next().unwrap_or(value_str);
 if let Ok(value) = value_part.parse::<u16>() {
 return Ok(MssPattern::Fixed(value));
 }
 }

 Ok(MssPattern::None)
}

/// parsed IP flag
fn parse_ip_flags(flags_str: &str) -> Result<IpFlags, P0f parsed Error> {
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
 // ignore not know flag
 }
 }
 }

 Ok(IpFlags {
 df,
 id_plus,
 id_minus,
 })
}

/// will P0fTcpSignature convert to TcpSignature (for match)
impl From<P0fTcpSignature> for TcpSignature {
 fn from(p0f_sig: P0fTcpSignature) -> Self {
 // from MSS patternExtractfixedvalue (if may)
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

 // from optionsorder in Extract Window Scale (if exists)
 let window _scale = if p0f_sig.options_order.contains(&TcpOptionType::WindowScale) {
 Some(7) // defaultvalue，actualshould from countpacket in Extract
 } else {
 None
 };

 // from window valuepatternExtractfixedvalue (if may)
 let window _size = match &p0f_sig. window _value {
 WindowSizePattern::Value(v) => *v,
 _ => 0, // wildcard or otherpattern
 };

 TcpSignature {
 id: p0f_sig.id,
 ttl: p0f_sig.initial_ttl,
 window _size,
 mss,
 window _scale,
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
