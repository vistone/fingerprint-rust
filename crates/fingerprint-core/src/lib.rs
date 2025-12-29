//! # fingerprint-core
//!
//! 核心类型和工具函数模块

pub mod types;
pub mod utils;
pub mod dicttls;

pub use types::{BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate};
pub use utils::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name,
    is_mobile_profile, random_choice, random_choice_string,
};
pub use dicttls::*;
