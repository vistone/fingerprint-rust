# 统一指纹生成示例输出

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## 演示 1: 不同操作系统的 TCP Profile

### Windows 10/11
```
TTL: 128
Window Size: 64240
MSS: 1460
Window Scale: 8
```

### macOS 13/14/15
```
TTL: 64
Window Size: 65535
MSS: 1460
Window Scale: 6
```

### Linux/Ubuntu/Debian
```
TTL: 64
Window Size: 65535
MSS: 1460
Window Scale: 7
```

## 演示 2: 统一指纹生成代码示例

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;

// Windows User-Agent
let windows_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let profile = generate_unified_fingerprint("chrome_135", windows_ua)?;

// 结果:
// - 浏览器指纹: Chrome-135
// - TCP Profile: TTL=128, Window Size=64240, MSS=1460, Window Scale=8
```

```rust
// Linux User-Agent
let linux_ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let profile = generate_unified_fingerprint("chrome_135", linux_ua)?;

// 结果:
// - 浏览器指纹: Chrome-135
// - TCP Profile: TTL=64, Window Size=65535, MSS=1460, Window Scale=7
```

```rust
// macOS User-Agent
let macos_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let profile = generate_unified_fingerprint("chrome_135", macos_ua)?;

// 结果:
// - 浏览器指纹: Chrome-135
// - TCP Profile: TTL=64, Window Size=65535, MSS=1460, Window Scale=6
```

## 演示 3: 实际运行效果

运行测试 `cargo test -p fingerprint-profiles --lib test_unified_fingerprint_generation` 的输出：

```
test profiles::tests::test_unified_fingerprint_generation ... ok
```

测试验证了：
- ✅ Windows User-Agent 生成 TTL=128, Window Size=64240 的 TCP Profile
- ✅ Linux User-Agent 生成 TTL=64, Window Size=65535 的 TCP Profile
- ✅ macOS User-Agent 生成 TTL=64, Window Size=65535 的 TCP Profile

## 演示 4: 指纹一致性验证

当生成统一指纹时，系统会自动验证：

1. **从 User-Agent 提取操作系统信息**
   - `Windows NT 10.0` → Windows
   - `Macintosh; Intel Mac OS X` → macOS
   - `X11; Linux` → Linux

2. **生成匹配的 TCP Profile**
   - Windows → TTL=128, Window Size=64240
   - macOS → TTL=64, Window Size=65535
   - Linux → TTL=64, Window Size=65535

3. **确保一致性**
   - 浏览器指纹（User-Agent）和 TCP 指纹（p0f）完全匹配
   - 避免被检测系统识别为异常

## 完整示例

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;
use fingerprint_headers::useragent::get_user_agent_by_profile_name;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 生成 User-Agent
    let user_agent = get_user_agent_by_profile_name("chrome_135")?;
    println!("User-Agent: {}", user_agent);
    
    // 2. 生成统一的指纹（浏览器指纹 + TCP 指纹）
    let profile = generate_unified_fingerprint("chrome_135", &user_agent)?;
    
    // 3. 验证 TCP Profile 已同步
    if let Some(tcp_profile) = profile.tcp_profile {
        println!("TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
    }
    
    Ok(())
}
```

## 输出示例

```
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36
TCP Profile:
  TTL: 128
  Window Size: 64240
  MSS: Some(1460)
  Window Scale: Some(8)
```
