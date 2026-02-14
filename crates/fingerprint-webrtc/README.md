# fingerprint-webrtc

WebRTC 指纹识别模块，通过分析 ICE 候选者、连接参数等进行识别。

## 功能特性

- ✅ ICE 候选者收集
- ✅ STUN 服务器识别
- ✅ 连接参数分析
- ✅ 媒体类型检测
- ✅ 编码器能力分析
- 🔧 可选的高级网络分析

## 快速开始

```rust
use fingerprint_webrtc::WebRtcFingerprint;

let webrtc_fp = WebRtcFingerprint::extract()?;
println!("ICE Candidates: {:?}", webrtc_fp.ice_candidates);
println!("Encoders: {:?}", webrtc_fp.encoders);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `WebRtcFingerprint` | WebRTC 指纹容器 |
| `IceCandidate` | ICE 候选者 |
| `Encoder` | 编码器信息 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── ice.rs          # ICE 分析
└── encoders.rs     # 编码器分析
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
