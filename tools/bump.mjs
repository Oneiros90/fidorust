#!/usr/bin/env node
/**
 * Single entry point for project version.
 *
 *   node tools/bump.mjs patch              # 0.1.2 → 0.1.3, commit + tag v0.1.3
 *   node tools/bump.mjs minor
 *   node tools/bump.mjs major
 *   node tools/bump.mjs 0.2.0              # set exact version
 *   node tools/bump.mjs 0.2.0 --push       # also git push branch + tag
 *   node tools/bump.mjs 0.2.0 --files-only # only rewrite files (CI)
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const SEMVER = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;
const WORKSPACE_CRATES = ['fidocad-core', 'fidocad-gpu', 'fidocad-tauri', 'fidocad-wasm'];
const VERSION_FILES = [
	'apps/ui/package.json',
	'apps/ui/package-lock.json',
	'apps/ui/src-tauri/tauri.conf.json',
	'Cargo.toml',
	'Cargo.lock'
];

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const flags = new Set(args.filter((a) => a.startsWith('--')));
const positional = args.filter((a) => !a.startsWith('--'));

if (flags.has('--help') || positional.length !== 1) {
	console.error(`Usage: node tools/bump.mjs <patch|minor|major|x.y.z> [--push] [--files-only]

Rewrites package.json, tauri.conf.json, Cargo.toml, Cargo.lock, package-lock.json.
Unless --files-only, also commits those files and creates git tag v<version>.`);
	process.exit(flags.has('--help') ? 0 : 1);
}

function read(rel) {
	return readFileSync(resolve(root, rel), 'utf8');
}

function write(rel, text) {
	writeFileSync(resolve(root, rel), text);
}

function patch(rel, re, replacement, label = rel) {
	const text = read(rel);
	if (!re.test(text)) {
		console.error(`Could not update ${label}`);
		process.exit(1);
	}
	write(rel, text.replace(re, replacement));
}

function currentVersion() {
	const m = read('apps/ui/package.json').match(/"version"\s*:\s*"([^"]+)"/);
	if (!m) {
		console.error('No version in apps/ui/package.json');
		process.exit(1);
	}
	return m[1];
}

function nextVersion(spec) {
	if (spec.startsWith('v') && SEMVER.test(spec.slice(1))) return spec.slice(1);
	if (SEMVER.test(spec)) return spec;
	if (!['patch', 'minor', 'major'].includes(spec)) {
		console.error(`Unknown version spec: ${spec}`);
		process.exit(1);
	}
	const cur = currentVersion();
	const m = cur.match(/^(\d+)\.(\d+)\.(\d+)/);
	if (!m) {
		console.error(`Cannot bump from ${cur}`);
		process.exit(1);
	}
	let [major, minor, patch] = m.slice(1).map(Number);
	if (spec === 'major') {
		major += 1;
		minor = 0;
		patch = 0;
	} else if (spec === 'minor') {
		minor += 1;
		patch = 0;
	} else {
		patch += 1;
	}
	return `${major}.${minor}.${patch}`;
}

function applyVersion(version) {
	patch(
		'apps/ui/package.json',
		/"version"\s*:\s*"[^"]+"/,
		`"version": "${version}"`
	);
	patch(
		'apps/ui/src-tauri/tauri.conf.json',
		/"version"\s*:\s*"[^"]+"/,
		`"version": "${version}"`
	);

	let npmLock = read('apps/ui/package-lock.json');
	let n = 0;
	npmLock = npmLock.replace(/"version"\s*:\s*"[^"]+"/g, (m) => {
		n += 1;
		return n <= 2 ? `"version": "${version}"` : m;
	});
	if (n < 2) {
		console.error('Could not update apps/ui/package-lock.json');
		process.exit(1);
	}
	write('apps/ui/package-lock.json', npmLock);

	patch(
		'Cargo.toml',
		/^version = "[^"]+"/m,
		`version = "${version}"`,
		'Cargo.toml [workspace.package]'
	);

	let lock = read('Cargo.lock');
	for (const name of WORKSPACE_CRATES) {
		const re = new RegExp(`(name = "${name}"\\r?\\n)version = "[^"]+"`);
		if (!re.test(lock)) {
			console.error(`Could not find ${name} in Cargo.lock`);
			process.exit(1);
		}
		lock = lock.replace(re, `$1version = "${version}"`);
	}
	write('Cargo.lock', lock);
}

function git(gitArgs) {
	const r = spawnSync('git', gitArgs, { cwd: root, encoding: 'utf8' });
	if (r.status !== 0) {
		const err = (r.stderr || r.stdout || '').trim();
		console.error(err || `git ${gitArgs.join(' ')} failed`);
		process.exit(r.status ?? 1);
	}
	return r;
}

const version = nextVersion(positional[0]);
applyVersion(version);
console.log(`Set version to ${version}`);

if (flags.has('--files-only')) process.exit(0);

const tag = `v${version}`;
const existing = spawnSync('git', ['rev-parse', '-q', '--verify', `refs/tags/${tag}`], {
	cwd: root,
	encoding: 'utf8'
});
if (existing.status === 0) {
	console.error(`Tag ${tag} already exists`);
	process.exit(1);
}

git(['add', '--', ...VERSION_FILES]);
git(['commit', '-m', `Bump version to ${version}.`]);
git(['tag', tag]);
console.log(`Committed and tagged ${tag}`);

if (flags.has('--push')) {
	git(['push', 'origin', 'HEAD']);
	git(['push', 'origin', tag]);
	console.log(`Pushed HEAD and ${tag}`);
} else {
	console.log(`Next: git push origin HEAD && git push origin ${tag}`);
}
