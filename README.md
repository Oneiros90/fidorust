# FidoRust

Porting di [FidoCAD 0.96](https://www.enetsystems.com/~lorenzo/fidocad.asp) in Rust + Tauri 2 + Svelte 5.

Versione web: [oneiros90.github.io/fidorust](https://oneiros90.github.io/fidorust/)

- Formato `.fcd` originale
- Canvas WebGL2: il documento resta in WASM, niente dump JSON delle primitive
- Desktop (Windows / macOS / Linux) e web
- i18n italiano/inglese, temi chiaro/scuro
- Librerie standard `stdlib` e `PCB` incluse

## Sviluppo

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cd apps/ui
npm install
npm run dev          # web su http://localhost:5173
npm run tauri dev    # desktop
```

## Test formato

```bash
cargo test -p fidocad-core
```

I sorgenti originali MFC stanno in `vendor/` (gitignored). Vedi `vendor/README.txt`.

## Release

Dal root del repo, uno script aggiorna `package.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock` e `package-lock.json`, poi fa commit e tag:

```bash
node tools/bump.mjs patch          # 0.1.2 → 0.1.3 + tag v0.1.3
node tools/bump.mjs minor
node tools/bump.mjs major
node tools/bump.mjs 0.2.0          # versione esatta
node tools/bump.mjs patch --push   # come sopra, e push di branch + tag
```

Il push del tag `v*` fa partire la build Windows / macOS / WASM e il deploy su GitHub Pages.
