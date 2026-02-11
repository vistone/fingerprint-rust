# 快速 Git 操作指南

> 针对 fingerprint-rust 项目的常用 Git 命令速查表

## 📌 查看状态和日志

### 查看当前状态
```bash
cd /home/stone/fingerprint-rust
git status
```

### 查看提交历史
```bash
# 查看最近 10 个提交
git log --oneline -10

# 查看详细的提交信息
git log -1

# 查看某个文件的历史
git log -p src/http_client/mod.rs
```

### 查看本地与远程的差异
```bash
# 看看本地比远程多了什么
git log origin/main...HEAD

# 看看远程比本地多了什么
git log HEAD...origin/main
```

---

## 🔄 同步和更新

### 获取远程最新代码
```bash
# 只获取信息，不合并
git fetch origin

# 获取特定分支
git fetch origin main
```

### 更新本地代码（合并）
```bash
# 拉取并合并
git pull origin main

# 拉取并 rebase
git pull --rebase origin main
```

### 更新本地代码（rebase）
```bash
# Rebase 到最新的远程 main
git rebase origin/main

# 如果遇到冲突，解决后
git rebase --continue

# 或者放弃 rebase
git rebase --abort
```

---

## 📝 提交和推送

### 查看未暂存的改动
```bash
# 查看工作区的修改
git diff

# 查看暂存区的修改
git diff --staged
```

### 暂存文件
```bash
# 暂存所有改动
git add -A

# 暂存特定文件
git add src/http_client/mod.rs

# 暂存特定目录
git add docs/
```

### 创建提交
```bash
# 创建提交
git commit -m "提交消息"

# 跳过 Git hooks（如果出现错误）
git commit --no-verify -m "提交消息"

# 修改最后一个提交
git commit --amend
```

### 推送到远程
```bash
# 推送到远程 main 分支
git push origin main

# 强制推送（谨慎使用）
git push -f origin main

# 推送所有分支
git push origin --all
```

---

## 🌿 分支操作

### 查看分支
```bash
# 查看本地分支
git branch

# 查看所有分支（包括远程）
git branch -a

# 查看当前分支的上游分支
git branch -vv
```

### 创建和切换分支
```bash
# 创建新分支
git branch feature/new-feature

# 创建并切换到新分支
git checkout -b feature/new-feature

# 从远程分支创建本地分支
git checkout --track origin/copilot/add-latest-browser-fingerprints
```

### 删除分支
```bash
# 删除本地分支
git branch -d feature/completed

# 强制删除（谨慎）
git branch -D feature/unwanted

# 删除远程分支
git push origin --delete feature/remote-branch
```

---

## 🔀 合并和 Rebase

### 合并分支
```bash
# 合并其他分支到当前分支
git merge feature/new-feature

# 看看是否有冲突
git status

# 解决冲突后提交
git add .
git commit -m "Merge: resolve conflicts"
```

### Rebase
```bash
# Rebase 到另一个分支
git rebase main

# 交互式 rebase（编辑提交历史）
git rebase -i HEAD~3

# 继续 rebase（解决冲突后）
git rebase --continue

# 放弃 rebase
git rebase --abort
```

---

## 🚨 撤销操作

### 撤销未暂存的改动
```bash
# 撤销工作区的改动
git restore <file>

# 或者用旧命令
git checkout -- <file>
```

### 撤销暂存的改动
```bash
# 取消暂存
git restore --staged <file>

# 或者用旧命令
git reset HEAD <file>
```

### 撤销已提交的改动
```bash
# 撤销最后一个提交（保留改动）
git reset --soft HEAD~1

# 撤销最后一个提交（删除改动）
git reset --hard HEAD~1

# 用 revert 创建一个反向提交
git revert HEAD
```

### 恢复已删除的文件
```bash
# 查看已删除的文件
git log --diff-filter=D --summary | grep delete

# 恢复已删除的文件
git checkout <commit>~1 -- <file>
```

---

## 🔍 搜索和检查

### 搜索提交
```bash
# 搜索提交消息
git log --grep="修复"

# 搜索作者
git log --author="stone"

# 搜索在特定时间范围的提交
git log --since="2026-02-01" --until="2026-02-11"
```

### 搜索代码
```bash
# 搜索代码中的内容
git log -S "search_text" --oneline

# 在已删除的代码中搜索
git log -p -S "deleted_function"
```

### 检查谁改动了什么
```bash
# 查看每一行代码的修改历史
git blame src/http_client/mod.rs

# 查看特定行范围的历史
git blame -L 100,200 src/http_client/mod.rs
```

---

## 📊 查看变化

### 比较差异
```bash
# 比较工作区和最后一个提交
git diff

# 比较暂存区和最后一个提交
git diff --staged

# 比较两个分支
git diff main feature/new-feature

# 比较两个提交
git diff abc123 def456
```

### 查看提交详情
```bash
# 查看某个提交的详细信息
git show abc123

# 查看某个文件在某个提交中的内容
git show abc123:path/to/file.rs

# 查看某个提交的变化统计
git show --stat abc123
```

---

## 🏷️ 标签操作

### 查看标签
```bash
# 列出所有标签
git tag

# 查看特定标签的信息
git show v2.1.0
```

### 创建标签
```bash
# 创建轻量级标签
git tag v2.1.1

# 创建带注释的标签
git tag -a v2.1.1 -m "版本 2.1.1 发布"

# 为某个提交创建标签
git tag v2.1.1 abc123
```

### 删除和推送标签
```bash
# 删除本地标签
git tag -d v2.1.1

# 删除远程标签
git push origin --delete v2.1.1

# 推送所有标签
git push origin --tags
```

---

## 🔐 安全和清理

### 清理本地仓库
```bash
# 删除未追踪的文件（预览）
git clean -fd --dry-run

# 删除未追踪的文件
git clean -fd

# 删除未追踪的文件和目录
git clean -fdx
```

### 压缩提交历史
```bash
# 压缩最后 3 个提交
git rebase -i HEAD~3

# 然后在编辑器中将要压缩的行改为 squash (s)
```

### 检查安全
```bash
# 检查仓库完整性
git fsck

# 查看引用日志（reflog）- 找回丢失的提交
git reflog

# 恢复丢失的提交
git checkout abc123
```

---

## 🤝 协作操作

### 查看他人的分支
```bash
# 获取所有人的分支
git fetch origin

# 切换到他人的分支
git checkout origin/copilot/add-latest-browser-fingerprints
```

### 处理冲突
```bash
# 查看冲突
git status

# 查看冲突的具体内容
git diff

# 解决冲突后
git add <resolved-file>
git commit -m "Merge: resolve conflicts"
```

### 创建 Pull Request
```bash
# 推送本地分支到远程
git push origin feature/new-feature

# 然后在 GitHub 上创建 Pull Request
```

---

## 📱 远程仓库管理

### 查看远程
```bash
# 列出所有远程
git remote

# 查看远程详情
git remote -v

# 查看某个远程的详细信息
git remote show origin
```

### 添加和删除远程
```bash
# 添加新远程
git remote add upstream https://github.com/other/fingerprint-rust.git

# 删除远程
git remote remove upstream

# 重命名远程
git remote rename old_name new_name
```

### 同步多个远程
```bash
# 从上游拉取
git fetch upstream

# 合并上游的改动
git merge upstream/main

# 推送到自己的远程
git push origin main
```

---

## 💡 有用的别名

添加到 `~/.gitconfig`:
```
[alias]
    st = status
    co = checkout
    br = branch
    ci = commit
    unstage = restore --staged
    last = log -1 HEAD
    visual = log --graph --oneline --all
    whoami = config user.name
    mylog = log --oneline -10
```

使用:
```bash
git st          # 等同于 git status
git co -b dev   # 等同于 git checkout -b dev
git mylog       # 查看最近 10 个提交
```

---

## 🐛 调试和故障排除

### 查看 Git 配置
```bash
# 查看所有配置
git config --list

# 查看本地仓库配置
git config --local --list

# 查看全局配置
git config --global --list
```

### 设置 Git 配置
```bash
# 设置用户信息
git config user.name "Your Name"
git config user.email "your@email.com"

# 设置全局配置
git config --global user.name "Your Name"
```

### 调试 Git 命令
```bash
# 启用 Git 的详细输出
GIT_TRACE=1 git push

# 调试 SSH 连接
GIT_TRACE_PERFORMANCE=1 git clone

# 查看 Git 的内部状态
git cat-file -p HEAD
```

---

## 📚 针对本项目的常见任务

### 任务 1: 拉取最新的远程代码
```bash
cd /home/stone/fingerprint-rust
git fetch origin
git rebase origin/main
```

### 任务 2: 创建新功能分支
```bash
git fetch origin
git checkout -b feature/my-feature origin/main
# 进行开发...
git push origin feature/my-feature
```

### 任务 3: 更新本地 main 分支
```bash
git checkout main
git pull origin main
```

### 任务 4: 查看远程的新分支
```bash
git fetch origin
git branch -a | grep copilot  # 查看所有 copilot 分支
```

### 任务 5: 推送本地提交
```bash
git push origin main
```

---

## 🔗 有用的资源

- [Git 官方文档](https://git-scm.com/doc)
- [GitHub 官方文档](https://docs.github.com)
- [Pro Git 书籍](https://git-scm.com/book/zh/v2)
- [GitHub 命令行工具 (gh)](https://cli.github.com/)

---

**最后更新**: 2026-02-11
**针对项目**: fingerprint-rust
**作用**: 日常 Git 操作快速参考


