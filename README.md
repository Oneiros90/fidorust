# FidoRust

FidoRust aims to be the spiritual successor to [FidoCAD](https://www.enetsystems.com/~lorenzo/fidocad.asp): a modern, capable, and efficient schematic editor available everywhere (desktop and web) without losing the simplicity and directness that made the original program legendary. Built with Rust, Tauri 2, and Svelte 5.

Heartfelt thanks to **Lorenzo Lutti**, the original author of FidoCAD, for a tool that shaped generations of hobbyists and professionals. My heartfelt thanks also to **Bruno Valente** for his advice, supervision, and testing.

Developed by **Lorenzo Valente** ([lorenzo.valente.my](https://lorenzo.valente.my))

Web app: [oneiros90.github.io/fidorust](https://oneiros90.github.io/fidorust/)

- Original `.fcd` format
- WebGL2 canvas: the document stays in WASM, no JSON dump of primitives
- Desktop (Windows / macOS / Linux) and web
- Italian/English i18n, light/dark themes
- Standard `stdlib` and `PCB` libraries included

## Development

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cd apps/ui
npm install
npm run dev          # web at http://localhost:5173
npm run tauri dev    # desktop
```