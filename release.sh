#!/usr/bin/env bash
#
# release.sh —— screen-hopper 一键发布脚本
#
# 做的事（按顺序）：
#   1. 校验版本号格式、检查 tag 没重复
#   2. 把 Cargo.toml 和 tauri.conf.json 的版本号改成目标版本
#   3. 构建免安装 exe（npm run tauri build -- --no-bundle）
#   4. 提交、打 tag、推送 main + tag（走 SSH）
#   5. 用 gh 发 GitHub Release 并挂上 exe（英文发版说明）
#
# 用法：
#   ./release.sh <版本号> [发版说明文件]
#
#   ./release.sh 0.3.1                  # 用默认英文说明
#   ./release.sh 0.3.1 notes.md         # 用 notes.md 作为发版说明
#
# 发版纪律：发版前请先在 CHANGELOG.md 顶部加好对应版本的中文变更记录。
#
set -euo pipefail

# ---- 配置 ----
REPO_SLUG="CodeWhatD/screen-hopper"
GIT_NAME="CodeWhatD"
GIT_EMAIL="1911650840@qq.com"
EXE_PATH="src-tauri/target/release/screen-hopper.exe"

# 把 cargo 和 gh 加进 PATH（这台机器上的位置）
export PATH="$HOME/.cargo/bin:/c/Program Files/GitHub CLI:$PATH"

# ---- 切到脚本所在目录（仓库根）----
cd "$(dirname "$0")"

# ---- 参数校验 ----
if [ $# -lt 1 ]; then
  echo "用法: ./release.sh <版本号> [发版说明文件]"
  echo "例如: ./release.sh 0.3.1"
  exit 1
fi

VERSION="$1"
NOTES_FILE="${2:-}"
TAG="v${VERSION}"

if ! echo "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "❌ 版本号格式不对：'$VERSION'，应形如 0.3.1"
  exit 1
fi

# ---- 依赖检查 ----
command -v cargo >/dev/null || { echo "❌ 找不到 cargo，请确认 Rust 已装"; exit 1; }
command -v gh    >/dev/null || { echo "❌ 找不到 gh，请确认 GitHub CLI 已装并 gh auth login 过"; exit 1; }

# ---- tag 不能重复 ----
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "❌ tag $TAG 已存在，换个版本号或先删掉旧 tag"
  exit 1
fi

echo "▶ 准备发布 $TAG"

# ---- 改版本号 ----
echo "▶ 更新 Cargo.toml / tauri.conf.json 版本号 → $VERSION"
sed -i '0,/^version = ".*"/s//version = "'"$VERSION"'"/' src-tauri/Cargo.toml
sed -i 's/"version": "[^"]*"/"version": "'"$VERSION"'"/'   src-tauri/tauri.conf.json

grep -q "version = \"$VERSION\"" src-tauri/Cargo.toml       || { echo "❌ Cargo.toml 版本号没改成功"; exit 1; }
grep -q "\"version\": \"$VERSION\"" src-tauri/tauri.conf.json || { echo "❌ tauri.conf.json 版本号没改成功"; exit 1; }

# ---- 构建 ----
echo "▶ 构建免安装 exe（可能要几分钟）…"
npm run tauri build -- --no-bundle

[ -f "$EXE_PATH" ] || { echo "❌ 没找到产物 $EXE_PATH，构建可能失败"; exit 1; }
echo "▶ 构建完成：$EXE_PATH ($(du -h "$EXE_PATH" | cut -f1))"

# ---- 提交 + tag + 推送 ----
echo "▶ 提交并打 tag"
git add -A
git -c user.name="$GIT_NAME" -c user.email="$GIT_EMAIL" \
    commit -m "release: $TAG"
git tag -a "$TAG" -m "$TAG"

echo "▶ 推送 main + tag（SSH）"
git push origin main
git push origin "$TAG"

# ---- 发 Release ----
echo "▶ 创建 GitHub Release"
ASSET="${EXE_PATH}#screen-hopper.exe (portable, double-click to run on Win10/11)"

if [ -n "$NOTES_FILE" ] && [ -f "$NOTES_FILE" ]; then
  gh release create "$TAG" "$ASSET" --title "screen-hopper $TAG" --notes-file "$NOTES_FILE"
else
  gh release create "$TAG" "$ASSET" --title "screen-hopper $TAG" --notes \
"## What's new
See [CHANGELOG.md](https://github.com/${REPO_SLUG}/blob/main/CHANGELOG.md) for details.

## Download
Grab \`screen-hopper.exe\` below, copy it to the controlled PC (Win10/11) and run it."
fi

echo ""
echo "✅ 发布完成：https://github.com/${REPO_SLUG}/releases/tag/${TAG}"
