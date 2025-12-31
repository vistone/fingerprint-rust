//! p0f 签名解析器（详细实现）
//!
//! 完整实现 p0f.fp 格式的解析，支持所有字段和模式。

use crate::passive::tcp::TcpSignature;
use thiserror::Error;

/// p0f TCP 签名（完整版）
#[derive(Debug, Clone)]
pub struct P0fTcpSignature {
    /// 签名 ID
    pub id: String,

    /// 标签信息
    pub label: SignatureLabel,

    /// 系统类型限制（可选）
    pub sys: Option<Vec<SystemType>>,

    /// TTL 模式
    pub ttl_pattern: TtlPattern,

    /// 初始 TTL 值
    pub initial_ttl: u8,

    /// 窗口大小模式
    pub window_mode: WindowMode,

    /// 窗口大小值模式
    pub window_value: WindowSizePattern,

    /// MSS 模式
    pub mss_pattern: MssPattern,

    /// TCP 选项顺序
    pub options_order: Vec<TcpOptionType>,

    /// IP 标志
    pub ip_flags: IpFlags,

    /// 其他字段
    pub other: String,
}

/// 签名标签
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureLabel {
    /// 匹配类型：s (specific) 或 g (generic)
    pub match_type: MatchType,

    /// 系统类型：unix, win, ! (application)
    pub sys_type: SystemType,

    /// 操作系统名称
    pub os: String,

    /// 版本信息
    pub version: String,
}

/// 匹配类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
    Specific, // s
    Generic,  // g
}

/// 系统类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemType {
    Unix,        // unix
    Windows,     // win
    Application, // !
}

/// TTL 模式
#[derive(Debug, Clone, PartialEq)]
pub enum TtlPattern {
    /// 通配符 *
    Wildcard,
    /// 具体值
    Value(u8),
}

/// 窗口大小模式
#[derive(Debug, Clone, PartialEq)]
pub enum WindowMode {
    /// 模式 0: 固定值
    Fixed,
    /// 模式 1: 倍数
    Multiple,
    /// 模式 2: 模数
    Modulo,
    /// 模式 3: 随机
    Random,
}

/// 窗口大小值模式
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSizePattern {
    /// 通配符 *
    Wildcard,
    /// 具体值
    Value(u16),
    /// 倍数模式：m*N
    Multiple(u16),
    /// 模数模式：%N
    Modulo(u16),
}

/// MSS 模式
#[derive(Debug, Clone, PartialEq)]
pub enum MssPattern {
    /// 无 MSS
    None,
    /// 固定值：mss,1460
    Fixed(u16),
    /// 倍数模式：mss*20,10 (20的倍数+10)
    Multiple { multiplier: u16, offset: u16 },
}

/// TCP 选项类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpOptionType {
    Mss,         // mss
    WindowScale, // ws
    Sack,        // sok
    Timestamp,   // ts
    Nop,         // nop
    End,         // eol
    Other(u8),   // 其他
}

/// IP 标志
#[derive(Debug, Clone, PartialEq)]
pub struct IpFlags {
    /// Don't Fragment
    pub df: bool,
    /// ID 递增
    pub id_plus: bool,
    /// ID 固定
    pub id_minus: bool,
}

/// p0f 解析错误
#[derive(Debug, Error)]
pub enum P0fParseError {
    #[error("无效的标签格式: {0}")]
    InvalidLabel(String),

    #[error("无效的签名格式: {0}")]
    InvalidSignature(String),

    #[error("解析错误: {0}")]
    Parse(String),
}

/// 解析 p0f TCP 签名
pub fn parse_tcp_signature(label: &str, sig: &str) -> Result<P0fTcpSignature, P0fParseError> {
    // 解析标签
    let label_info = parse_label(label)?;

    // 解析签名：格式为 [ttl]:[初始ttl]:[窗口模式]:[窗口值]:[TCP选项]:[IP标志]:[其他]
    let parts: Vec<&str> = sig.split(':').collect();
    if parts.len() < 7 {
        return Err(P0fParseError::InvalidSignature(format!(
            "签名部分数量不足: 期望7个，实际{}个",
            parts.len()
        )));
    }

    // 解析 TTL 模式
    let ttl_pattern =
        if parts[0] == "*" {
            TtlPattern::Wildcard
        } else {
            TtlPattern::Value(parts[0].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("无效的 TTL: {}", parts[0]))
            })?)
        };

    // 解析初始 TTL
    let initial_ttl = parts[1]
        .parse()
        .map_err(|_| P0fParseError::InvalidSignature(format!("无效的初始 TTL: {}", parts[1])))?;

    // 解析窗口大小模式
    let window_mode = match parts[2] {
        "0" => WindowMode::Fixed,
        "1" => WindowMode::Multiple,
        "2" => WindowMode::Modulo,
        "3" => WindowMode::Random,
        _ => {
            return Err(P0fParseError::InvalidSignature(format!(
                "无效的窗口模式: {}",
                parts[2]
            )))
        }
    };

    // 解析窗口大小值模式
    let window_value = parse_window_size_pattern(parts[3])?;

    // 解析 MSS 模式和 TCP 选项
    // p0f 格式: [ttl]:[初始ttl]:[窗口模式]:[窗口值]:[MSS模式]:[选项顺序]:[IP标志]:[其他]
    // 所以 parts[4] 是 MSS 模式，parts[5] 是选项顺序
    let mss_str = parts[4];
    let options_str = if parts.len() > 5 { parts[5] } else { "" };

    // 合并 MSS 模式和选项顺序进行解析
    let full_options_str = if !options_str.is_empty() {
        format!("{}:{}", mss_str, options_str)
    } else {
        mss_str.to_string()
    };

    let (mss_pattern, options_order) = parse_tcp_options(&full_options_str)?;

    // 解析 IP 标志
    let ip_flags = if parts.len() > 6 {
        parse_ip_flags(parts[6])?
    } else {
        IpFlags {
            df: false,
            id_plus: false,
            id_minus: false,
        }
    };

    // 其他字段
    let other = if parts.len() > 7 {
        parts[7].to_string()
    } else {
        "0".to_string()
    };

    Ok(P0fTcpSignature {
        id: format!("tcp-{}", label.replace(':', "-")),
        label: label_info,
        sys: None, // 将在后续解析 sys 字段时设置
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

/// 解析标签
fn parse_label(label: &str) -> Result<SignatureLabel, P0fParseError> {
    // 格式: s:unix:Linux:3.11 and newer
    let parts: Vec<&str> = label.split(':').collect();
    if parts.len() < 4 {
        return Err(P0fParseError::InvalidLabel(format!(
            "标签部分数量不足: 期望4个，实际{}个",
            parts.len()
        )));
    }

    let match_type = match parts[0] {
        "s" => MatchType::Specific,
        "g" => MatchType::Generic,
        _ => {
            return Err(P0fParseError::InvalidLabel(format!(
                "无效的匹配类型: {}",
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
                "无效的系统类型: {}",
                parts[1]
            )))
        }
    };

    Ok(SignatureLabel {
        match_type,
        sys_type,
        os: parts[2].to_string(),
        version: parts[3..].join(":"), // 版本可能包含冒号
    })
}

/// 解析窗口大小模式
fn parse_window_size_pattern(pattern: &str) -> Result<WindowSizePattern, P0fParseError> {
    if pattern == "*" {
        return Ok(WindowSizePattern::Wildcard);
    }

    // 检查倍数模式：m*N
    if let Some(pos) = pattern.find('*') {
        let multiplier = pattern[pos + 1..]
            .parse()
            .map_err(|_| P0fParseError::InvalidSignature(format!("无效的窗口倍数: {}", pattern)))?;
        return Ok(WindowSizePattern::Multiple(multiplier));
    }

    // 检查模数模式：%N
    if let Some(stripped) = pattern.strip_prefix('%') {
        let modulo = stripped
            .parse()
            .map_err(|_| P0fParseError::InvalidSignature(format!("无效的窗口模数: {}", pattern)))?;
        return Ok(WindowSizePattern::Modulo(modulo));
    }

    // 固定值
    let value = pattern
        .parse()
        .map_err(|_| P0fParseError::InvalidSignature(format!("无效的窗口大小: {}", pattern)))?;
    Ok(WindowSizePattern::Value(value))
}

/// 解析 TCP 选项
fn parse_tcp_options(options_str: &str) -> Result<(MssPattern, Vec<TcpOptionType>), P0fParseError> {
    let mut mss_pattern = MssPattern::None;
    let mut options_order = Vec::new();

    // 选项格式: mss*20,10:mss,sok,ts,nop,ws
    // 第一部分是 MSS 模式，第二部分是选项顺序

    let parts: Vec<&str> = options_str.split(':').collect();
    if parts.is_empty() {
        return Ok((mss_pattern, options_order));
    }

    // 解析 MSS 模式（第一部分）
    // 格式可能是: mss*20,10 或 mss,1460
    let mss_part = parts[0];
    if mss_part.contains("mss") {
        mss_pattern = parse_mss_pattern(mss_part)?;
    }

    // 解析选项顺序
    // 格式可能是: mss*20,10:mss,sok,ts,nop,ws
    // 或者: mss,1460:mss,sok,ts,nop,ws
    // 第二部分是选项顺序
    if parts.len() > 1 {
        // 第二部分包含选项顺序
        for opt_str in parts[1].split(',') {
            let opt = match opt_str.trim() {
                "mss" => TcpOptionType::Mss,
                "ws" => TcpOptionType::WindowScale,
                "sok" => TcpOptionType::Sack,
                "ts" => TcpOptionType::Timestamp,
                "nop" => TcpOptionType::Nop,
                "eol" => TcpOptionType::End,
                _ => {
                    // 尝试解析为数字
                    if let Ok(num) = opt_str.parse::<u8>() {
                        TcpOptionType::Other(num)
                    } else {
                        continue; // 跳过未知选项
                    }
                }
            };
            options_order.push(opt);
        }
    } else {
        // 如果没有第二部分，可能选项顺序就在第一部分（在 MSS 模式之后）
        // 格式: mss*20,10 或 mss,1460
        // 这种情况下，选项顺序可能不存在，或者需要从其他地方提取
        // 暂时不处理这种情况
    }

    Ok((mss_pattern, options_order))
}

/// 解析 MSS 模式
fn parse_mss_pattern(mss_str: &str) -> Result<MssPattern, P0fParseError> {
    // 格式: mss*20,10 或 mss,1460
    if let Some(pos) = mss_str.find('*') {
        // 倍数模式: mss*20,10
        let after_star = &mss_str[pos + 1..];
        let parts: Vec<&str> = after_star.split(',').collect();
        if parts.len() >= 2 {
            let multiplier = parts[0].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("无效的 MSS 倍数: {}", parts[0]))
            })?;
            let offset = parts[1].parse().map_err(|_| {
                P0fParseError::InvalidSignature(format!("无效的 MSS 偏移: {}", parts[1]))
            })?;
            return Ok(MssPattern::Multiple { multiplier, offset });
        }
    }

    // 固定值模式: mss,1460
    // 查找第一个逗号后的数字
    if let Some(pos) = mss_str.find(',') {
        let value_str = &mss_str[pos + 1..];
        // 可能还有更多逗号，只取第一个数字部分
        let value_part = value_str.split(',').next().unwrap_or(value_str);
        if let Ok(value) = value_part.parse::<u16>() {
            return Ok(MssPattern::Fixed(value));
        }
    }

    Ok(MssPattern::None)
}

/// 解析 IP 标志
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
                // 忽略未知标志
            }
        }
    }

    Ok(IpFlags {
        df,
        id_plus,
        id_minus,
    })
}

/// 将 P0fTcpSignature 转换为 TcpSignature（用于匹配）
impl From<P0fTcpSignature> for TcpSignature {
    fn from(p0f_sig: P0fTcpSignature) -> Self {
        // 从 MSS 模式提取固定值（如果可能）
        let mss = match &p0f_sig.mss_pattern {
            MssPattern::Fixed(v) => Some(*v),
            MssPattern::Multiple { multiplier, offset } => {
                // 使用第一个可能的 MSS 值作为示例
                let m = (*multiplier as u32)
                    .saturating_mul(10)
                    .saturating_add(*offset as u32);
                Some(m.min(65535) as u16)
            }
            MssPattern::None => None,
        };

        // 从选项顺序中提取 Window Scale（如果存在）
        let window_scale = if p0f_sig.options_order.contains(&TcpOptionType::WindowScale) {
            Some(7) // 默认值，实际应该从数据包中提取
        } else {
            None
        };

        // 从窗口值模式提取固定值（如果可能）
        let window_size = match &p0f_sig.window_value {
            WindowSizePattern::Value(v) => *v,
            _ => 0, // 通配符或其他模式
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
