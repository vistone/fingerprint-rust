# 参考文档

fingerprint-rust的完整参考文档。

## 📖 内容

### 技术规范
技术实现细节和规范可以在[technical/](technical/)中找到：

- **[GREASE规范化](technical/GREASE_NORMALIZATION.md)** - TLS GREASE处理
- **[HPACK指纹识别](technical/HPACK_FINGERPRINTING.md)** - HTTP/2头压缩指纹识别
- **[数据包捕获实现](technical/PACKET_CAPTURE_IMPLEMENTATION.md)** - 网络数据包捕获和分析
- **[PSK 0-RTT实现](technical/PSK_0RTT_IMPLEMENTATION.md)** - 预共享密钥和0-RTT恢复
- **[RustLS指纹集成](technical/RUSTLS_FINGERPRINT_INTEGRATION.md)** - RustLS TLS库集成
- **[TCP握手指纹识别](technical/TCP_HANDSHAKE_FINGERPRINTING.md)** - TCP级别指纹分析
- **[TLS ClientHello集成](technical/TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md)** - ClientHello消息处理
- **[TTL评分优化](technical/TTL_SCORING_OPTIMIZATION.md)** - 生存时间值优化

### 工具文档
- **[文档管理工具](document-management-tools.md)** - 管理项目文档的工具

## 📚 使用本参考

- **关于协议实现细节** → 查看技术规范文件夹
- **关于API使用** → 参见[user-guides/](../user-guides/)
- **关于架构细节** → 参见[ARCHITECTURE.md](../ARCHITECTURE.md)
- **关于模块文档** → 参见[modules/](../modules/)

---

**最后更新**: 2026-02-14  
**状态**: 完整参考
