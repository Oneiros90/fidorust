#!/usr/bin/env bash
# Copy every fidorust-pro.bin GitHub Release asset into dist/pro/<version>/.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dest="${1:-"$root/apps/ui/dist/pro"}"
repo="${GITHUB_REPOSITORY:-Oneiros90/fidorust}"

mkdir -p "$dest"

if ! command -v gh >/dev/null; then
	echo "gh CLI is required" >&2
	exit 1
fi

mapfile -t tags < <(gh release list --repo "$repo" --limit 200 --json tagName --jq '.[].tagName')

staged=0
for tag in "${tags[@]}"; do
	[[ "$tag" == v* ]] || continue
	ver="${tag#v}"
	dir="$dest/$ver"
	mkdir -p "$dir"
	if gh release download "$tag" --repo "$repo" --pattern 'fidorust-pro.bin' --dir "$dir" --clobber >/dev/null 2>&1; then
		echo "staged $tag -> pro/$ver/fidorust-pro.bin"
		staged=$((staged + 1))
	else
		rmdir "$dir" 2>/dev/null || rm -rf "$dir"
	fi
done

echo "staged $staged pro module(s) into $dest"
