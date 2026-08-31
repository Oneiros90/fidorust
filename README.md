# FidoRust

Porting di [FidoCAD 0.96](https://www.enetsystems.com/~lorenzo/fidocad.asp) in Rust + Tauri 2 + Svelte 5.

- Formato `.fcd` originale (estensioni FidoCadJ `FCJ`/`FJC`/`CV`/`CP` ignorate in lettura)
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
npm run dev          # web su http://localhost:1420
npm run tauri dev    # desktop
```

## Test formato

```bash
cargo test -p fidocad-core
```

I sorgenti originali MFC stanno in `vendor/` (gitignored). Vedi `vendor/README.txt`.
