//! HTTP/2 configuremoduletesting

use fingerprint::*;

#[test]
fn test_chrome_http2_settings() {
    let (settings, settings_order) = chrome_http2_settings();

    // validate settings 不to空
    assert!(!settings.is_empty());

    // validateincludeallrequiredofset
    assert!(settings.contains_key(&HTTP2SettingID::HeaderTableSize.as_u16()));
    assert!(settings.contains_key(&HTTP2SettingID::EnablePush.as_u16()));
    assert!(settings.contains_key(&HTTP2SettingID::MaxConcurrentStreams.as_u16()));
    assert!(settings.contains_key(&HTTP2SettingID::InitialWindowSize.as_u16()));
    assert!(settings.contains_key(&HTTP2SettingID::MaxFrameSize.as_u16()));
    assert!(settings.contains_key(&HTTP2SettingID::MaxHeaderListSize.as_u16()));

    // validate顺序
    assert_eq!(settings_order.len(), 6);
    assert_eq!(settings_order[0], HTTP2SettingID::HeaderTableSize.as_u16());

    // validate具体值
    assert_eq!(
        settings.get(&HTTP2SettingID::HeaderTableSize.as_u16()),
        Some(&65536)
    );
    assert_eq!(settings.get(&HTTP2SettingID::EnablePush.as_u16()), Some(&0));
}

#[test]
fn test_firefox_http2_settings() {
    let (settings, settings_order) = firefox_http2_settings();

    assert!(!settings.is_empty());
    assert_eq!(settings_order.len(), 6);

    // Firefox of InitialWindowSize 与 Chrome 不同
    let firefox_window = settings
        .get(&HTTP2SettingID::InitialWindowSize.as_u16())
        .unwrap();
    let (chrome_settings, _) = chrome_http2_settings();
    let chrome_window = chrome_settings
        .get(&HTTP2SettingID::InitialWindowSize.as_u16())
        .unwrap();
    assert_ne!(firefox_window, chrome_window);
}

#[test]
fn test_safari_http2_settings() {
    let (settings, settings_order) = safari_http2_settings();

    assert!(!settings.is_empty());
    assert_eq!(settings_order.len(), 6);

    // Safari of MaxConcurrentStreams 与 Chrome 不同
    let safari_streams = settings
        .get(&HTTP2SettingID::MaxConcurrentStreams.as_u16())
        .unwrap();
    assert_eq!(safari_streams, &100);
}

#[test]
fn test_chrome_pseudo_header_order() {
    let order = chrome_pseudo_header_order();
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], ":method");
    assert_eq!(order[1], ":authority");
    assert_eq!(order[2], ":scheme");
    assert_eq!(order[3], ":path");
}

#[test]
fn test_firefox_pseudo_header_order() {
    let order = firefox_pseudo_header_order();
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], ":method");
    assert_eq!(order[1], ":path");
    assert_eq!(order[2], ":authority");
    assert_eq!(order[3], ":scheme");

    // Firefox and Chrome of顺序不同
    let chrome_order = chrome_pseudo_header_order();
    assert_ne!(order, chrome_order);
}

#[test]
fn test_safari_pseudo_header_order() {
    let order = safari_pseudo_header_order();
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], ":method");
    assert_eq!(order[1], ":scheme");
    assert_eq!(order[2], ":path");
    assert_eq!(order[3], ":authority");

    // Safari of顺序与 Chrome and Firefox 都不同
    let chrome_order = chrome_pseudo_header_order();
    let firefox_order = firefox_pseudo_header_order();
    assert_ne!(order, chrome_order);
    assert_ne!(order, firefox_order);
}

#[test]
fn test_chrome_connection_flow() {
    assert_eq!(CHROME_CONNECTION_FLOW, 15663105);
}

#[test]
fn test_chrome_header_priority() {
    let priority = chrome_header_priority();
    assert_eq!(priority.weight, 255);
    assert_eq!(priority.stream_dependency, 0);
    assert!(!priority.exclusive);
}

#[test]
fn test_http2_setting_id_as_u16() {
    assert_eq!(HTTP2SettingID::HeaderTableSize.as_u16(), 1);
    assert_eq!(HTTP2SettingID::EnablePush.as_u16(), 2);
    assert_eq!(HTTP2SettingID::MaxConcurrentStreams.as_u16(), 3);
    assert_eq!(HTTP2SettingID::InitialWindowSize.as_u16(), 4);
    assert_eq!(HTTP2SettingID::MaxFrameSize.as_u16(), 5);
    assert_eq!(HTTP2SettingID::MaxHeaderListSize.as_u16(), 6);
    assert_eq!(HTTP2SettingID::EnableConnectProtocol.as_u16(), 8);
}

#[test]
fn test_http2_priority_param() {
    let param = HTTP2PriorityParam::new(128, 0, true);
    assert_eq!(param.weight, 128);
    assert_eq!(param.stream_dependency, 0);
    assert!(param.exclusive);
}

#[test]
fn test_different_browsers_have_different_settings() {
    let (chrome, _) = chrome_http2_settings();
    let (firefox, _) = firefox_http2_settings();
    let (safari, _) = safari_http2_settings();

    // validate不同浏览器of InitialWindowSize 不同
    let chrome_window = chrome
        .get(&HTTP2SettingID::InitialWindowSize.as_u16())
        .unwrap();
    let firefox_window = firefox
        .get(&HTTP2SettingID::InitialWindowSize.as_u16())
        .unwrap();
    let safari_window = safari
        .get(&HTTP2SettingID::InitialWindowSize.as_u16())
        .unwrap();

    assert_ne!(chrome_window, firefox_window);
    assert_ne!(chrome_window, safari_window);
    assert_ne!(firefox_window, safari_window);
}
