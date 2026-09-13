#!/usr/bin/env bash
# Copy every Pro GitHub Release asset into dist/pro/<version>/.
# Pages gets both fidorust-pro.bin (release name) and fidorust-pro.dat (loader URL).
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
	tmp="$(mktemp -d)"
	src=""
	if gh release download "$tag" --repo "$repo" --pattern 'fidorust-pro.bin' --dir "$tmp" --clobber >/dev/null 2>&1 \
		&& [[ -f "$tmp/fidorust-pro.bin" ]]; then
		src="$tmp/fidorust-pro.bin"
	elif gh release download "$tag" --repo "$repo" --pattern 'fidorust-pro.dat' --dir "$tmp" --clobber >/dev/null 2>&1 \
		&& [[ -f "$tmp/fidorust-pro.dat" ]]; then
		src="$tmp/fidorust-pro.dat"
	fi
	if [[ -n "$src" ]]; then
		mkdir -p "$dir"
		cp "$src" "$dir/fidorust-pro.bin"
		cp "$src" "$dir/fidorust-pro.dat"
		echo "staged $tag -> pro/$ver/fidorust-pro.{bin,dat}"
		staged=$((staged + 1))
	fi
	rm -rf "$tmp"
done

echo "staged $staged pro module(s) into $dest"
