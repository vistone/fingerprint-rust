//! 针对 `https://kh.google.com/rt/earth/PlanetoidMetadata` 的“全指纹/全协议”实网回归测试
//!
//! 目标：
//! - 把 `profiles::mapped_tls_clients()` 里所有 browser profile 都跑一遍
//! - 每个 profile 分别用 HTTP/1.1、HTTP/2、HTTP/3 访问同一个地址
//! - 验证：能连通、能解析响应、协议版本符合预期
//!
//! 注意：
//! - 这是实网测试，默认 `#[ignore]`，需要用 `-- --ignored` 显式执行
//! - 为了避免并发压测导致的偶发失败，这里用“单测试函数串行循环”的方式执行

use std::time::Instant;

use fingerprint::http_client::{request::HttpMethod, HttpRequest};
use fingerprint::profiles::mapped_tls_clients;
use fingerprint::{get_user_agent_by_profile_name, HttpClientConfig};

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";
const TEST_HOST: &str = "kh.google.com";
const TEST_PORT: u16 = 443;
const TEST_PATH: &str = "/rt/earth/PlanetoidMetadata";

fn run_http1(profile_name: &str, profile: &fingerprint::ClientProfile) -> Result<(), String> {
    let ua = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| "Mozilla/5.0".into());
    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };
    let request = HttpRequest::new(HttpMethod::Get, TEST_URL);
    let resp = fingerprint::http_client::tls::send_https_request(
        TEST_HOST, TEST_PORT, TEST_PATH, &request, &config,
    )
    .map_err(|e| format!("{e}"))?;
    if !resp.http_version.contains("HTTP/1.1") {
        return Err(format!("预期 HTTP/1.1，实际 {}", resp.http_version));
    }
    if resp.status_code != 200 {
        return Err(format!("预期 200，实际 {}", resp.status_code));
    }
    if resp.body.is_empty() {
        return Err("响应 body 为空".to_string());
    }
    Ok(())
}

#[cfg(feature = "http2")]
fn run_http2(profile_name: &str, profile: &fingerprint::ClientProfile) -> Result<(), String> {
    let ua = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| "Mozilla/5.0".into());
    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };
    let request = HttpRequest::new(HttpMethod::Get, TEST_URL);
    let resp = fingerprint::http_client::http2::send_http2_request(
        TEST_HOST, TEST_PORT, TEST_PATH, &request, &config,
    )
    .map_err(|e| format!("{e}"))?;
    if !resp.http_version.contains("HTTP/2") {
        return Err(format!("预期 HTTP/2，实际 {}", resp.http_version));
    }
    if resp.status_code != 200 {
        return Err(format!("预期 200，实际 {}", resp.status_code));
    }
    if resp.body.is_empty() {
        return Err("响应 body 为空".to_string());
    }
    Ok(())
}

#[cfg(feature = "http3")]
fn run_http3(profile_name: &str, profile: &fingerprint::ClientProfile) -> Result<(), String> {
    let ua = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| "Mozilla/5.0".into());
    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http3: true,
        // 避免 h3 失败时静默回退到 h2，从而“看起来成功但不是 h3”
        prefer_http2: false,
        ..Default::default()
    };
    let request = HttpRequest::new(HttpMethod::Get, TEST_URL);
    let resp = fingerprint::http_client::http3::send_http3_request(
        TEST_HOST, TEST_PORT, TEST_PATH, &request, &config,
    )
    .map_err(|e| format!("{e}"))?;
    if !resp.http_version.contains("HTTP/3") {
        return Err(format!("预期 HTTP/3，实际 {}", resp.http_version));
    }
    if resp.status_code != 200 {
        return Err(format!("预期 200，实际 {}", resp.status_code));
    }
    if resp.body.is_empty() {
        return Err("响应 body 为空".to_string());
    }
    Ok(())
}

#[test]
#[ignore]
fn test_kh_google_planetoidmetadata_all_fingerprints_all_protocols() {
    let start = Instant::now();
    let clients = mapped_tls_clients();

    let mut total = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for (name, profile) in clients.iter() {
        // HTTP/1.1
        total += 1;
        if let Err(e) = run_http1(name, profile) {
            failed.push(format!("{name} / http1: {e}"));
        }

        // HTTP/2
        #[cfg(feature = "http2")]
        {
            total += 1;
            if let Err(e) = run_http2(name, profile) {
                failed.push(format!("{name} / http2: {e}"));
            }
        }

        // HTTP/3
        #[cfg(feature = "http3")]
        {
            total += 1;
            if let Err(e) = run_http3(name, profile) {
                failed.push(format!("{name} / http3: {e}"));
            }
        }
    }

    eprintln!(
        "kh.google.com PlanetoidMetadata 全指纹/全协议：总用例 {}，失败 {}，耗时 {:?}",
        total,
        failed.len(),
        start.elapsed()
    );

    if !failed.is_empty() {
        eprintln!("失败明细（前 50 条）：");
        for line in failed.iter().take(50) {
            eprintln!("  - {}", line);
        }
        panic!("存在失败用例：{} / {}", failed.len(), total);
    }
}

/// 连接池版本：全指纹 × (HTTP/1.1+pool / HTTP/2+pool / HTTP/3+pool)
///
/// 说明：
/// - 这是你要求的“最重要的连接池”覆盖
/// - 这里走的都是 *pool 实现*，并严格检查响应版本（不会偷跑 fallback）
#[tokio::test]
#[cfg(all(feature = "connection-pool", feature = "http2", feature = "http3"))]
#[ignore]
async fn test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool() {
    use fingerprint::http_client::{
        http2_pool, http3_pool, pool::ConnectionPoolManager, PoolManagerConfig,
    };
    use std::sync::Arc;
    use std::time::Duration;

    let start = Instant::now();
    let clients = mapped_tls_clients();
    // 这个测试会创建大量连接（所有 profile × 3 协议）。
    // 在实网环境中，HTTP/2/3 的驱动任务可能导致连接无法及时归还，从而触发 GetConnectionTimeout。
    // 为了让“全量覆盖”稳定执行，这里显式提高 max_connections，并放宽超时。
    let pool_cfg = PoolManagerConfig {
        max_connections: 1200,
        min_idle: 0,
        connect_timeout: Duration::from_secs(60),
        idle_timeout: Duration::from_secs(120),
        max_lifetime: Duration::from_secs(600),
        enable_reuse: true,
    };
    let pool_manager = Arc::new(ConnectionPoolManager::new(pool_cfg));

    let mut total = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for (name, profile) in clients.iter() {
        let ua = get_user_agent_by_profile_name(name).unwrap_or_else(|_| "Mozilla/5.0".to_string());

        let config = HttpClientConfig {
            user_agent: ua,
            profile: Some(profile.clone()),
            // pool 测试里协议由具体函数决定，这里 prefer_* 仅用于 TLS customizer 注入
            prefer_http2: true,
            prefer_http3: true,
            ..Default::default()
        };

        let request = HttpRequest::new(HttpMethod::Get, TEST_URL);

        // HTTP/1.1 over TLS + pool
        total += 1;
        match fingerprint::http_client::tls::send_https_request_with_pool(
            TEST_HOST,
            TEST_PORT,
            TEST_PATH,
            &request,
            &config,
            &pool_manager,
        ) {
            Ok(resp) => {
                if !resp.http_version.contains("HTTP/1.1")
                    || resp.status_code != 200
                    || resp.body.is_empty()
                {
                    failed.push(format!(
                        "{name} / http1_pool: version={} status={} body={}",
                        resp.http_version,
                        resp.status_code,
                        resp.body.len()
                    ));
                }
            }
            Err(e) => failed.push(format!("{name} / http1_pool: {e}")),
        }

        // HTTP/2 + pool
        total += 1;
        match http2_pool::send_http2_request_with_pool(
            TEST_HOST,
            TEST_PORT,
            TEST_PATH,
            &request,
            &config,
            &pool_manager,
        )
        .await
        {
            Ok(resp) => {
                if !resp.http_version.contains("HTTP/2")
                    || resp.status_code != 200
                    || resp.body.is_empty()
                {
                    failed.push(format!(
                        "{name} / http2_pool: version={} status={} body={}",
                        resp.http_version,
                        resp.status_code,
                        resp.body.len()
                    ));
                }
            }
            Err(e) => failed.push(format!("{name} / http2_pool: {e}")),
        }

        // HTTP/3 + pool
        total += 1;
        match http3_pool::send_http3_request_with_pool(
            TEST_HOST,
            TEST_PORT,
            TEST_PATH,
            &request,
            &config,
            &pool_manager,
        )
        .await
        {
            Ok(resp) => {
                if !resp.http_version.contains("HTTP/3")
                    || resp.status_code != 200
                    || resp.body.is_empty()
                {
                    failed.push(format!(
                        "{name} / http3_pool: version={} status={} body={}",
                        resp.http_version,
                        resp.status_code,
                        resp.body.len()
                    ));
                }
            }
            Err(e) => failed.push(format!("{name} / http3_pool: {e}")),
        }
    }

    eprintln!(
        "kh.google.com PlanetoidMetadata 全指纹/全协议(连接池)：总用例 {}，失败 {}，耗时 {:?}",
        total,
        failed.len(),
        start.elapsed()
    );

    if !failed.is_empty() {
        eprintln!("失败明细（前 50 条）：");
        for line in failed.iter().take(50) {
            eprintln!("  - {}", line);
        }
        panic!("存在失败用例：{} / {}", failed.len(), total);
    }
}
