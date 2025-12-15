# Google Earth API 测试可执行程序

## 可执行文件位置

```
target/release/examples/test_google_earth
```

## 编译

```bash
# 编译 release 版本（推荐，性能更好）
cargo build --example test_google_earth --features rustls-tls,http2,http3 --release

# 编译 debug 版本（用于调试）
cargo build --example test_google_earth --features rustls-tls,http2,http3
```

编译后的可执行文件位于：
- Release: `target/release/examples/test_google_earth`
- Debug: `target/debug/examples/test_google_earth`

## 使用方法

### 查看帮助

```bash
./target/release/examples/test_google_earth --help
```

### 测试 HTTP/1.1（所有 66 个指纹）

```bash
./target/release/examples/test_google_earth http1
```

### 测试 HTTP/2（所有 66 个指纹）

```bash
./target/release/examples/test_google_earth http2
```

**注意**: 需要编译时启用 `http2` feature

### 测试 HTTP/3（所有 66 个指纹）

```bash
./target/release/examples/test_google_earth http3
```

**注意**: 需要编译时启用 `http3` feature

### 全面测试（所有协议）

```bash
./target/release/examples/test_google_earth all
```

这会测试所有 66 个浏览器指纹 × 3 个协议 = 198 个测试用例。

## 使用 cargo run（无需单独编译）

也可以直接使用 `cargo run`，无需先编译：

```bash
# 测试 HTTP/1.1
cargo run --example test_google_earth --features rustls-tls,http2,http3 --release -- http1

# 测试 HTTP/2
cargo run --example test_google_earth --features rustls-tls,http2,http3 --release -- http2

# 测试 HTTP/3
cargo run --example test_google_earth --features rustls-tls,http2,http3 --release -- http3

# 全面测试
cargo run --example test_google_earth --features rustls-tls,http2,http3 --release -- all
```

## 输出示例

```
╔══════════════════════════════════════════════════════════╗
║  Google Earth API 全面测试 - HTTP/1.1                    ║
║  地址: https://kh.google.com/rt/earth/PlanetoidMetadata  ║
╚══════════════════════════════════════════════════════════╝

🔍 测试所有 66 个浏览器指纹 (HTTP/1.1)...

  [ 1/66] chrome_103                          ... ✅ 200 (450ms)
  [ 2/66] chrome_133                          ... ✅ 200 (452ms)
  ...

╔══════════════════════════════════════════════════════════╗
║  HTTP/1.1 测试结果汇总                        ║
╚══════════════════════════════════════════════════════════╝

  总测试数: 66
  成功: 66 ✅
  失败: 0 ❌
  成功率: 100.0%
  总耗时: 45.23s
```

## 文件大小

Release 版本可执行文件大小约 **5.5MB**（包含所有依赖）。

## 优势

相比使用 `cargo test`，可执行文件的优势：
- ✅ 无需每次重新编译
- ✅ 可以独立分发
- ✅ 可以添加到 PATH 中直接运行
- ✅ 性能更好（release 优化）

## 复制到系统路径（可选）

```bash
# 复制到 /usr/local/bin（需要 sudo）
sudo cp target/release/examples/test_google_earth /usr/local/bin/

# 之后可以直接运行
test_google_earth http1
```

