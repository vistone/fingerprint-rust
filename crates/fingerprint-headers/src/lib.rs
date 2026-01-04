//! # fingerprint-headers
//!
//! HTTP Headers and User-Agent generation module

pub mod headers;
pub mod http2_config;
pub mod useragent;

pub use headers::{generate_headers, random_language, HTTPHeaders};
pub use http2_config::{
    chrome_header_order, chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_header_order, firefox_http2_settings, firefox_pseudo_header_order, safari_header_order,
    safari_http2_settings, safari_pseudo_header_order, HTTP2Priority, HTTP2PriorityParam,
    HTTP2SettingID, HTTP2Settings,
};
pub use useragent::{
    get_user_agent_by_profile_name, get_user_agent_by_profile_name_with_os, random_os,
    UserAgentGenerator,
};
