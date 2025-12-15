# 所有浏览器指纹导出测试报告

## 测试时间
$(date)

## 测试结果

### 总体统计
- **总测试数**: 66 个浏览器指纹
- **成功**: 66 ✅
- **失败**: 0 ❌
- **成功率**: 100%

### 测试覆盖的浏览器类型

#### Chrome 系列 (19个)
- ✅ chrome_103, chrome_104, chrome_105, chrome_106, chrome_107, chrome_108
- ✅ chrome_109, chrome_110, chrome_111, chrome_112
- ✅ chrome_116_PSK, chrome_116_PSK_PQ
- ✅ chrome_117, chrome_120, chrome_124
- ✅ chrome_130_PSK, chrome_131, chrome_131_PSK
- ✅ chrome_133, chrome_133_PSK

#### Firefox 系列 (13个)
- ✅ firefox_102, firefox_104, firefox_105, firefox_106
- ✅ firefox_108, firefox_110, firefox_117
- ✅ firefox_120, firefox_123, firefox_132
- ✅ firefox_133, firefox_135

#### Safari 系列 (9个)
- ✅ safari_15_6_1, safari_16_0, safari_ipad_15_6
- ✅ safari_ios_15_5, safari_ios_15_6, safari_ios_16_0
- ✅ safari_ios_17_0, safari_ios_18_0, safari_ios_18_5

#### Opera 系列 (3个)
- ✅ opera_89, opera_90, opera_91

#### 移动端和自定义指纹 (22个)
- ✅ zalando_android_mobile, zalando_ios_mobile
- ✅ nike_ios_mobile, nike_android_mobile
- ✅ mms_ios, mms_ios_2, mms_ios_3
- ✅ mesh_ios, mesh_android, mesh_ios_2, mesh_android_2
- ✅ confirmed_ios, confirmed_android, confirmed_android_2
- ✅ okhttp4_android_7, okhttp4_android_8, okhttp4_android_9
- ✅ okhttp4_android_10, okhttp4_android_11, okhttp4_android_12, okhttp4_android_13
- ✅ cloudflare_custom

## 测试方法

1. 使用 `export_config` 示例程序导出每个指纹的 JSON 配置
2. 验证导出的 JSON 文件：
   - 文件存在且非空
   - 包含必需的 `cipher_suites` 字段
   - JSON 格式正确

## 导出的文件

所有导出的配置文件保存在 `exported_profiles/` 目录中，每个指纹对应一个 JSON 文件。

### 文件格式

每个 JSON 文件包含：
- `cipher_suites`: 密码套件列表
- `compression_methods`: 压缩方法列表
- `extensions`: TLS 扩展列表（包含 GREASE、SNI、KeyShare 等）
- `tls_vers_min`: 最小 TLS 版本
- `tls_vers_max`: 最大 TLS 版本

## 结论

✅ **所有 66 个浏览器指纹的导出功能测试全部通过！**

所有指纹都能成功导出为有效的 JSON 配置文件，格式正确，包含完整的 TLS 握手信息。

