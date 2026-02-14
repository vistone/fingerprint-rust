# 项目规制

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 项目规制

---

## 文件放置

- 禁止在仓库根目录新增文件。
- 文档放在 `docs/`，配置放在 `config/`，脚本放在 `scripts/`，数据放在 `data/` 或 `dataset/`。
- 翻译草稿和记录必须放在 `docs/translation-notes/`（已被 git 忽略）。

## 文档双语对齐

- 所有面向用户的文档必须同时存在于 `docs/en/` 与 `docs/zh/`，路径和文件名保持一致。
- 更新任一语言时，必须在同一次变更中更新另一语言。
- `docs/` 下新增文档必须放在 `docs/en/` 与 `docs/zh/`，不得直接放在 `docs/` 根目录。

## 执行与校验

- 提交前运行 `python3 scripts/verify_doc_pairs.py` 或 `./scripts/pre_commit_test.sh`。
- CI 或提交前检查若违反规制将失败。

## 例外

- 任何例外必须获得维护者批准，并在拉取请求中说明。
