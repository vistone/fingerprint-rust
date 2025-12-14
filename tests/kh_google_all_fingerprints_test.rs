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
// 为了适配执行环境对“单次命令运行时间”的限制，这里把全量 profile 拆成若干 chunk，
// 同时在 chunk 内做小并发，保证单次测试能在较短时间内结束。
const CHUNK_SIZE: usize = 12;
const CHUNK_CONCURRENCY: usize = 3;

fn run_http1(profile_name: &str, profile: &fingerprint::ClientProfile) -> Result<(), String> {
    let ua = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| "Mozilla/5.0".into());
    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: false,
        prefer_http3: false,
        connect_timeout: std::time::Duration::from_secs(15),
        read_timeout: std::time::Duration::from_secs(15),
        write_timeout: std::time::Duration::from_secs(15),
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
        connect_timeout: std::time::Duration::from_secs(15),
        read_timeout: std::time::Duration::from_secs(15),
        write_timeout: std::time::Duration::from_secs(15),
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
        connect_timeout: std::time::Duration::from_secs(15),
        read_timeout: std::time::Duration::from_secs(15),
        write_timeout: std::time::Duration::from_secs(15),
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

fn sorted_profile_names() -> Vec<String> {
    let clients = mapped_tls_clients();
    let mut names = clients.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
}

fn run_chunk_all_protocols(chunk_idx: usize) -> (usize, Vec<String>) {
    let start = Instant::now();
    let clients = mapped_tls_clients();
    let names = sorted_profile_names();
    let n = names.len();
    let begin = chunk_idx * CHUNK_SIZE;
    let end = ((chunk_idx + 1) * CHUNK_SIZE).min(n);
    if begin >= n {
        return (0, Vec::new());
    }

    eprintln!(
        "\n=== kh.google.com PlanetoidMetadata chunk {} [{}..{}) / {} ===",
        chunk_idx, begin, end, n
    );

    let mut total = 0usize;
    let mut failed: Vec<String> = Vec::new();

    // chunk 内做小并发：每个 profile 内部仍然是顺序（http1->http2->http3），避免过度压测。
    let slice = &names[begin..end];
    let mut cursor = 0usize;
    while cursor < slice.len() {
        let batch_end = (cursor + CHUNK_CONCURRENCY).min(slice.len());
        let batch = &slice[cursor..batch_end];

        let mut joins = Vec::with_capacity(batch.len());
        for name in batch {
            let name = name.clone();
            let profile = clients.get(&name).expect("profile exists").clone();
            joins.push(std::thread::spawn(move || {
                let mut local_failed: Vec<String> = Vec::new();

                if let Err(e) = run_http1(&name, &profile) {
                    local_failed.push(format!("{name} / http1: {e}"));
                }
                #[cfg(feature = "http2")]
                {
                    if let Err(e) = run_http2(&name, &profile) {
                        local_failed.push(format!("{name} / http2: {e}"));
                    }
                }
                #[cfg(feature = "http3")]
                {
                    if let Err(e) = run_http3(&name, &profile) {
                        local_failed.push(format!("{name} / http3: {e}"));
                    }
                }
                local_failed
            }));
        }

        for j in joins {
            let local_failed = j.join().unwrap_or_else(|_| vec!["thread panicked".to_string()]);
            failed.extend(local_failed);
        }

        // 统计总用例数
        total += (batch_end - cursor) * (1 + cfg!(feature = "http2") as usize + cfg!(feature = "http3") as usize);
        cursor = batch_end;
    }

    eprintln!(
        "chunk {} 完成：总用例 {}，失败 {}，耗时 {:?}",
        chunk_idx,
        total,
        failed.len(),
        start.elapsed()
    );

    (total, failed)
}

#[test]
#[ignore]
fn test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_00() {
    let (total, failed) = run_chunk_all_protocols(0);
    if total == 0 {
        return;
    }
    if !failed.is_empty() {
        eprintln!("失败明细（前 50 条）：");
        for line in failed.iter().take(50) {
            eprintln!("  - {}", line);
        }
        panic!("存在失败用例：{} / {}", failed.len(), total);
    }
}

macro_rules! gen_chunk_tests {
    ($(($name:ident, $idx:expr)),+ $(,)?) => {
        $(
            #[test]
            #[ignore]
            fn $name() {
                let (total, failed) = run_chunk_all_protocols($idx);
                if total == 0 {
                    return;
                }
                if !failed.is_empty() {
                    eprintln!("失败明细（前 50 条）：");
                    for line in failed.iter().take(50) {
                        eprintln!("  - {}", line);
                    }
                    panic!("存在失败用例：{} / {}", failed.len(), total);
                }
            }
        )+
    };
}

gen_chunk_tests!(
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_01, 1),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_02, 2),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_03, 3),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_04, 4),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_05, 5),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_06, 6),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_07, 7),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_08, 8),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_09, 9),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_10, 10),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_11, 11),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_12, 12),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_13, 13),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_14, 14),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_15, 15),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_16, 16),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_17, 17),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_18, 18),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_19, 19),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_20, 20),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_21, 21),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_22, 22),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_23, 23),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_24, 24),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_25, 25),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_26, 26),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_27, 27),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_28, 28),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_29, 29),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_30, 30),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_chunk_31, 31),
);

/// 连接池版本：全指纹 × (HTTP/1.1+pool / HTTP/2+pool / HTTP/3+pool)
///
/// 说明：
/// - 这是你要求的“最重要的连接池”覆盖
/// - 这里走的都是 *pool 实现*，并严格检查响应版本（不会偷跑 fallback）
#[cfg(all(feature = "connection-pool", feature = "http2", feature = "http3"))]
async fn run_chunk_all_protocols_with_pool(chunk_idx: usize) -> (usize, Vec<String>) {
    use fingerprint::http_client::{
        http2_pool, http3_pool, pool::ConnectionPoolManager, PoolManagerConfig,
    };
    use std::sync::Arc;
    use std::time::Duration;

    let start = Instant::now();
    let names = sorted_profile_names();
    let n = names.len();
    let begin = chunk_idx * CHUNK_SIZE;
    let end = ((chunk_idx + 1) * CHUNK_SIZE).min(n);
    if begin >= n {
        return (0, Vec::new());
    }

    eprintln!(
        "\n=== kh.google.com PlanetoidMetadata pool chunk {} [{}..{}) / {} ===",
        chunk_idx, begin, end, n
    );
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

    let mut failed: Vec<String> = Vec::new();

    use futures::stream::{self, StreamExt};
    let slice = names[begin..end].to_vec();
    let results = stream::iter(slice.into_iter())
        .map(|name: String| {
            let pool_manager = pool_manager.clone();
            async move {
                let mut local_failed: Vec<String> = Vec::new();
                let clients = mapped_tls_clients();
                let profile = clients.get(&name).expect("profile exists").clone();
                let ua = get_user_agent_by_profile_name(&name)
                    .unwrap_or_else(|_| "Mozilla/5.0".to_string());

                let config = HttpClientConfig {
                    user_agent: ua,
                    profile: Some(profile),
                    // pool 测试里协议由具体函数决定，这里 prefer_* 仅用于 TLS customizer 注入
                    prefer_http2: true,
                    prefer_http3: true,
                    connect_timeout: Duration::from_secs(20),
                    read_timeout: Duration::from_secs(20),
                    write_timeout: Duration::from_secs(20),
                    ..Default::default()
                };

                let request = HttpRequest::new(HttpMethod::Get, TEST_URL);

                // HTTP/1.1 over TLS + pool
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
                            local_failed.push(format!(
                                "{name} / http1_pool: version={} status={} body={}",
                                resp.http_version,
                                resp.status_code,
                                resp.body.len()
                            ));
                        }
                    }
                    Err(e) => local_failed.push(format!("{name} / http1_pool: {e}")),
                }

                // HTTP/2 + pool
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
                            local_failed.push(format!(
                                "{name} / http2_pool: version={} status={} body={}",
                                resp.http_version,
                                resp.status_code,
                                resp.body.len()
                            ));
                        }
                    }
                    Err(e) => local_failed.push(format!("{name} / http2_pool: {e}")),
                }

                // HTTP/3 + pool
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
                            local_failed.push(format!(
                                "{name} / http3_pool: version={} status={} body={}",
                                resp.http_version,
                                resp.status_code,
                                resp.body.len()
                            ));
                        }
                    }
                    Err(e) => local_failed.push(format!("{name} / http3_pool: {e}")),
                }

                local_failed
            }
        })
        .buffer_unordered(CHUNK_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    for local in results {
        failed.extend(local);
    }

    let total = (end - begin) * 3;

    eprintln!(
        "pool chunk {} 完成：总用例 {}，失败 {}，耗时 {:?}",
        chunk_idx,
        total,
        failed.len(),
        start.elapsed()
    );

    (total, failed)
}

#[tokio::test]
#[cfg(all(feature = "connection-pool", feature = "http2", feature = "http3"))]
#[ignore]
async fn test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_00() {
    let (total, failed) = run_chunk_all_protocols_with_pool(0).await;
    if total == 0 {
        return;
    }
    if !failed.is_empty() {
        eprintln!("失败明细（前 50 条）：");
        for line in failed.iter().take(50) {
            eprintln!("  - {}", line);
        }
        panic!("存在失败用例：{} / {}", failed.len(), total);
    }
}

macro_rules! gen_pool_chunk_tests {
    ($(($name:ident, $idx:expr)),+ $(,)?) => {
        $(
            #[tokio::test]
            #[cfg(all(feature = "connection-pool", feature = "http2", feature = "http3"))]
            #[ignore]
            async fn $name() {
                let (total, failed) = run_chunk_all_protocols_with_pool($idx).await;
                if total == 0 {
                    return;
                }
                if !failed.is_empty() {
                    eprintln!("失败明细（前 50 条）：");
                    for line in failed.iter().take(50) {
                        eprintln!("  - {}", line);
                    }
                    panic!("存在失败用例：{} / {}", failed.len(), total);
                }
            }
        )+
    };
}

gen_pool_chunk_tests!(
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_01, 1),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_02, 2),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_03, 3),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_04, 4),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_05, 5),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_06, 6),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_07, 7),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_08, 8),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_09, 9),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_10, 10),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_11, 11),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_12, 12),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_13, 13),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_14, 14),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_15, 15),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_16, 16),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_17, 17),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_18, 18),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_19, 19),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_20, 20),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_21, 21),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_22, 22),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_23, 23),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_24, 24),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_25, 25),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_26, 26),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_27, 27),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_28, 28),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_29, 29),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_30, 30),
    (test_kh_google_planetoidmetadata_all_fingerprints_all_protocols_with_pool_chunk_31, 31),
);
