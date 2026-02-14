# 📚 fingerprint-rust 文档维护规则

**版本**: 1.0  
**最后更新**: 2026-02-14  
**适用范围**: docs/ 目录下的所有 Markdown 文档

---

## 📖 目录

1. [基本原则](#基本原则)
2. [中英文同步规则](#中英文同步规则)
3. [术语使用标准](#术语使用标准)
4. [文档结构规范](#文档结构规范)
5. [新建文档流程](#新建文档流程)
6. [审查检查清单](#审查检查清单)
7. [常见问题解决](#常见问题解决)

---

## 基本原则

### 1. 双语对称性 🔄
```
    原则: docs/en/ 和 docs/zh/ 必须保持结构完全一致
    
    ✅ 正确:
    docs/en/API.md          ↔ docs/zh/API.md
    docs/en/guides/         ↔ docs/zh/guides/
    (所有文件都有对应的中文版本)
    
    ❌ 错误:
    docs/en/ 中有文件，但 docs/zh/ 中没有对应文件
```

### 2. 语言纯正性 🌍
```
    原则: 每个版本的文档应该用该语言纯正地表达
    
    ✅ 英文版本 (en/):
    - 所有内容用英文
    - 技术术语保持一致
    - 代码注释用英文
    
    ✅ 中文版本 (zh/):
    - 所有内容用中文
    - 重要术语在括号内跟英文: 浏览器指纹 (Browser Fingerprinting)
    - 代码注释保持英文（代码逻辑不翻译）
```

### 3. 准确性优先 💯
```
    原则: 翻译准确性 > 格式一致性 > 完美修辞
    
    优先级顺序:
    1. 技术准确性（必须）
    2. 术语一致性（必须）
    3. 格式规范（应该）
    4. 语言优雅（可选）
```

---

## 中英文同步规则

### 规则 1: 最小对称结构

```
每个版本必须有以下结构:

docs/en/                          docs/zh/
├── README.md               ↔     ├── README.md
├── INDEX.md                ↔     ├── INDEX.md
├── API.md                  ↔     ├── API.md
├── ARCHITECTURE.md         ↔     ├── ARCHITECTURE.md
├── CONTRIBUTING.md         ↔     ├── CONTRIBUTING.md
├── SECURITY.md             ↔     ├── SECURITY.md
├── CHANGELOG.md            ↔     ├── CHANGELOG.md
├── ORGANIZATION.md         ↔     ├── ORGANIZATION.md
│
├── user-guides/            ↔     ├── user-guides/
│   ├── README.md           ↔     │   ├── README.md
│   ├── getting-started.md  ↔     │   ├── getting-started.md
│   ├── api-usage.md        ↔     │   ├── api-usage.md
│   └── fingerprint-guide.md↔     │   └── fingerprint-guide.md
│
├── developer-guides/       ↔     ├── developer-guides/
│   └── [7 files]           ↔     │   └── [7 files]
│
├── guides/                 ↔     ├── guides/
│   └── [8 files]           ↔     │   └── [8 files]
│
├── modules/                ↔     ├── modules/
│   └── [13 files]          ↔     │   └── [13 files]
│
├── reference/              ↔     ├── reference/
│   └── [2 files]           ↔     │   └── [2 files]
│
├── architecture/           ↔     ├── architecture/
│   └── [5 files]           ↔     │   └── [5 files]
│
├── http-client/            ↔     ├── http-client/
│   └── [3 files]           ↔     │   └── [3 files]
│
└── security/               ↔     └── security/
    └── [3 files]           ↔         └── [3 files]
```

### 规则 2: 同步新增文档

```
当需要新增文档时:

步骤 1: 在 en/ 中创建文档
        env/new-feature/guide.md

步骤 2: 立即在 zh/ 中创建对应文档
        zh/new-feature/guide.md

步骤 3: 翻译内容（参考术语词典）

步骤 4: 在两个版本中都更新 README.md 和 INDEX.md

步骤 5: Git 提交时，确保两个文件都被提交:
        git add docs/en/new-feature/guide.md
        git add docs/zh/new-feature/guide.md
```

### 规则 3: 同步文件删除

```
当需要删除文档时:

步骤 1: 同时从 en/ 和 zh/ 中删除
        
步骤 2: 更新导航索引 (README.md, INDEX.md)

步骤 3: 或者移动到 archives/ 目录保留历史记录

规则: 永远不要只删除一个版本的文档
```

---

## 术语使用标准

### 保留为英文的术语（不翻译）
```
基础设施:
- TLS, SSL, HTTPS
- HTTP/1.1, HTTP/2, HTTP/3, QUIC
- ClientHello, ServerHello
- Cipher Suite, HPACK, GREASE

Rust 生态:
- Rust, Cargo, Crate, Workspace, Module
- rustls, tokio, ring, quinn

互联网标准:
- DNS, TCP, UDP, RTT, TTL
- API, SDK, CLI, REST, JSON

项目特定:
- fingerprint-rust (项目名)
- Fingerprinting (行为，保持英文)
```

### 必须翻译的术语（中英对照）
```
中文版本应使用以下标准翻译:

浏览器指纹识别 (Browser Fingerprinting)
被动识别 (Passive Identification)
主动防御 (Active Defense)
性能优化 (Performance Optimization)
安全审计 (Security Audit)
连接池 (Connection Pool)
威胁检测 (Threat Detection)
模块 (Module)
配置 (Configuration)
部署 (Deployment)
```

### 格式规范
```
✅ 正确格式:

英文版本:
"The TLS Handshake is part of the ClientHello mechanism"

中文版本:
"TLS 握手是 ClientHello 机制的一部分"
或
"TLS 握手 (TLS Handshake) 是 ClientHello 机制的一部分"

✅ 列表中的混合术语:
- 浏览器指纹识别 (Browser Fingerprinting) - 用于被动识别
- 连接池 (Connection Pool) - 管理网络连接

❌ 避免:
- 过度翻译技术术语
- 混合格式的术语定义
- 不一致的术语使用
```

---

## 文档结构规范

### 标题层级规范
```
# 主标题 (一级标题 - 只在文档开头使用一次)
## 主要章节 (二级标题)
### 小节 (三级标题)
#### 详细项目 (四级标题 - 如需要)
```

### 文档开头模板
```markdown
# [文档标题]

**版本**: [版本号]  
**最后更新**: [日期]  
**文档类型**: [类型: 技术参考/用户指南/开发指南/等]

---

## 目录
[可选，用于长文档]

---

[主要内容]
```

### 导航链接规范
```
✅ 正确:
- [API 参考](../API.md)
- [快速开始](./getting-started.md)
- [示例代码](../../examples/)

❌ 避免:
- [跨越语言版本的链接] ../zh/API.md
- [绝对路径] /docs/API.md
```

---

## 新建文档流程

### 流程图
```
需求 → 规划 → 中文初稿 → 英文初稿 → 审查 → 合并 → 发布

关键检查点:
1. 中文和英文同时创建（不能先有一个版本）
2. 术语检查（参考术语词典）
3. 链接验证（确保相对链接正确）
4. 格式检查（遵循规范模板）
5. 导航更新（README.md, INDEX.md）
```

### 具体步骤

```
步骤 1: 规划
  [ ] 确定文档标题（中英文相同概念）
  [ ] 确定类型（指南/参考/教程）
  [ ] 确定目录位置
  [ ] 检查是否需要多个子文档

步骤 2: 编写内容
  [ ] 中文版本 (zh/...)
  [ ] 英文版本 (en/...)
  [ ] 两个版本使用同一概念结构

步骤 3: 应用术语
  [ ] 检查所有技术术语
  [ ] 使用术语词典中的标准翻译
  [ ] 格式: 中文术语 (English Term)

步骤 4: 更新导航
  [ ] 更新 zh/README.md
  [ ] 更新 en/README.md
  [ ] 更新 zh/INDEX.md
  [ ] 更新 en/INDEX.md
  [ ] 如适用，更新子目录 README.md

步骤 5: Git 操作
  [ ] git add docs/zh/new-file.md
  [ ] git add docs/en/new-file.md
  [ ] git add docs/zh/README.md （如有修改）
  [ ] git add docs/en/README.md （如有修改）
  [ ] git commit -m "docs: add new documentation [zh/en]"

步骤 6: 审查
  [ ] 技术准确性审查
  [ ] 翻译准确性审查
  [ ] 链接有效性检查
  [ ] 格式一致性检查
```

---

## 审查检查清单

### 提交前检查清单 ✅

```
[ ] 中英文文件同时存在
[ ] 文件结构保持一致
[ ] 所有标题已翻译
[ ] 代码块保持完整（注释翻译）
[ ] 链接路径正确
[ ] 导航文件已更新
[ ] 术语使用一致
[ ] 格式符合规范
[ ] 没有混合的中英文段落（应该分语言）
[ ] 文件名一致 (除了内容语言外)
```

### 常见错误检查 ❌

```
[ ] 不要存在"英文版本中有，中文版本没有"的文件
[ ] 不要直接复制英文内容到中文版本
[ ] 不要翻译代码块中的代码
[ ] 不要为建立双语链接使用跨版本链接
[ ] 不要在一个文件中混合两种语言
[ ] 不要使用不一致的术语
[ ] 不要忘记更新导航索引
```

---

## 常见问题解决

### Q1: 代码注释应该如何处理？
```
英文版本: 保留英文注释
中文版本: 翻译注释为中文

示例:

// EN version
pub fn get_fingerprint() {
    // Generate random fingerprint
    ...
}

// ZH version
pub fn get_fingerprint() {
    // 生成随机指纹
    ...
}
```

### Q2: 如何处理外部链接？
```
保持 URL 不变，只翻译链接文本

✅ 正确:
[Rust 官方网站](https://www.rust-lang.org/)
[官方网站 (Official Website)](https://www.rust-lang.org/)

❌ 错误:
URL 不要翻译，即使是 URL 一部分
```

### Q3: 类似 "HTTP/2" 这样的混合术语如何处理？
```
保持原样，不要翻译：

✅ HTTP/2, HTTP/3, RFC 7230 等不需要翻译
中文文档: "HTTP/2 支持"
英文文档: "HTTP/2 support"
```

### Q4: 何时需要更新 archives/?
```
当文档被归档时:
- 旧版本指南 → archives/historical-guides/
- 完成的项目 → archives/project-docs/
- 发布的报告 → archives/published-reports/

规则: 
- 总是同时归档中英文版本
- 在两个版本中都删除旧文件引用
- 在 archives/ 中创建统一访问点
```

### Q5: 如何进行翻译质量评审？
```
建议的审查流程:

1. 自动检查: 术语词典验证
2. 人工审查: 技术准确性（由领域专家）
3. 语言审查: 语言流畅性（由 Native Speaker）
4. 最终检查: 格式和链接

频率: 
- 新文档: 发布前必须审查
- 更新: 重大更改需要审查
- 定期: 每季度进行一次全面审查
```

---

## 维护检查表

### 月度检查 📅
```
[ ] 验证 en/ 和 zh/ 目录结构一致
[ ] 检查是否有非翻译文件
[ ] 验证所有链接有效
[ ] 确认术语使用一致
[ ] 更新过期信息（版本号、日期等）
```

### 季度检查 📊
```
[ ] 审查所有新增文档的质量
[ ] 更新术语词典（如有新术语）
[ ] 检查翻译的一致性趋势
[ ] 收集用户反馈并改进
[ ] 更新此维护指南
```

### 年度检查 📈
```
[ ] 全面翻译质量评估
[ ] 分析常见错误并制定改进方案
[ ] 更新文档架构（如需要）
[ ] 制定下一年的维护计划
[ ] 分享经验和最佳实践
```

---

## 推荐的工具和流程

### 1. 自动检查工具
```bash
# 检查文件对称性
find docs/en -name "*.md" | while read f; do
    zh_file="${f/en/zh}"
    [ ! -f "$zh_file" ] && echo "Missing: $zh_file"
done

# 术语一致性检查
grep -r "某术语" docs/en/ docs/zh/ | sort | uniq -c
```

### 2. 版本控制实践
```
每个文档更新都应该：
- 提及影响的版本 (EN/ZH/Both)
- 清楚说明更改内容
- 保留更改历史

提交消息格式:
docs: [type] [description] [lang]

示例:
docs: fix API terminology [zh/EN]
docs: add new guide for TLS fingerprinting [en/zh]
docs: update version info [both]
```

### 3. 协作工作流
```
如果有多个翻译人员：

1. 分配责任: 谁负责 en/, 谁负责 zh/
2. 定期同步: 每周检查同步状态
3. 术语审查: 共享术语词典，定期更新
4. 知识分享: 月度会议分享最佳实践
```

---

## 相关资源

- [术语词典](terminology_dictionary.json)
- [翻译质量分析](translation_quality_analysis.md)
- [GitHub 项目](https://github.com/vistone/fingerprint-rust)

---

**维护负责人**: fingerprint-rust 文档团队  
**最后审核**: 2026-02-14  
**下次审核**: 2026-05-14

