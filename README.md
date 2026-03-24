# Cheshire CAT

**Cheshire CAT** is a cross-platform Computer-Assisted Translation (CAT) tool built from the ground up with Rust and Tauri 2. It is the spiritual successor to [Felix CAT](https://github.com/arktos-ai-lab/felix-cat-classic) — carrying forward the same translator-first philosophy into a modern, cross-platform architecture.

> The name is a deliberate double meaning: the Cheshire Cat from *Alice in Wonderland* — known for its enigmatic grin and tendency to appear and disappear — and **CAT**, for Computer-Assisted Translation. Just as Felix was a wink at the cartoon cat, Cheshire CAT is a wink at its famous cousin.

## Status

**Early developer preview (v0.0.1).** Core translation memory, AI-assisted suggestions, and QA checks are functional. Office add-in integration is under active development.

## What it does

- **Translation memory** — fuzzy and exact matching with SQLite backend, fast even on large TMs
- **AI-assisted suggestions** — works with local Ollama models (3B–7B, CPU-friendly for corporate laptops) or OpenAI-compatible APIs. LLM is only invoked when there is TM evidence to synthesise — not as a replacement for human translation
- **QA checks** — tag consistency, number consistency, flagged inline
- **Multi-format import** — XLIFF, plain text; Word/Excel/PowerPoint via Office integration (in progress)
- **Cross-platform** — Windows and macOS from the same codebase

## Why Rust + Tauri?

The original Felix is a Windows-only C++ application from 2013. Cheshire CAT keeps the translator-first workflow but replaces the platform-specific layer with Rust (fast, safe, cross-platform) and Tauri 2 (native shell, small bundle, no Electron overhead). The translation memory engine, segmenter, AI orchestration, and QA checks all live in independent Rust crates that can be tested, reused, and audited independently.

## Architecture

```
cheshire-cat/
├── src-tauri/          Tauri shell + Tauri commands (Rust)
├── src/                React + TypeScript frontend
├── crates/
│   ├── cheshire-tm/    Translation memory engine (SQLite via sqlx)
│   ├── cheshire-ai/    AI orchestration (Ollama / OpenAI)
│   ├── cheshire-qa/    QA checks (tags, numbers)
│   ├── cheshire-segmenter/  Text segmentation
│   └── cheshire-formats/    File format import/export
└── office-addin/       Office.js add-in (Word/Excel/PowerPoint)
```

## Getting started (development)

**Requirements:** Rust stable, Node.js 20+, npm.

```bash
git clone https://github.com/arktos-ai-lab/cheshire-cat.git
cd cheshire-cat
npm install
npm run tauri dev
```

For AI features, install [Ollama](https://ollama.ai) and pull a model:
```bash
ollama pull llama3.2:3b
```

## Building a release

```bash
npm run tauri build
```

Produces a signed installer in `src-tauri/target/release/bundle/`.

## AI model recommendations

Cheshire CAT is designed for corporate laptops — no GPU assumed.

| Model | VRAM / RAM | Speed (CPU) | Notes |
|-------|-----------|-------------|-------|
| `llama3.2:3b` | ~2 GB | ~10 tok/s | Best for limited hardware |
| `phi4-mini:3.8b` | ~2.5 GB | ~8 tok/s | Strong reasoning |
| `qwen2.5:7b-q4` | ~4.5 GB | ~5 tok/s | Best quality at 7B |

The AI is invoked in the background while the translator reads the source segment — latency is hidden by design.

## Relationship to Felix

| | Felix Original | Felix Classic | Cheshire CAT |
|--|----------------|---------------|--------------|
| Architecture | C++ / Win32 / COM | C++ / Win32 / COM | Rust / Tauri / React |
| Platform | Windows only | Windows only | Windows + macOS |
| Office integration | COM add-ins | COM add-ins (x86 + x64) | Office.js add-in |
| AI features | None | None | Yes (local + API) |
| Status | Archived (2015) | Maintained | Active development |

## Contributing

Contributions are welcome. The codebase is structured so that each crate has its own tests — please add tests for any new logic.

```bash
cargo test --workspace   # all tests
```

## License

MIT License — see [LICENSE](LICENSE).

Cheshire CAT is maintained by Ernst van Gassen under the same MIT license that covered Felix.
Original Felix CAT by Ryan Ginstrom, 1999–2015 — archived at [felix-cat-original](https://github.com/arktos-ai-lab/felix-cat-original) · maintained fork at [felix-cat-classic](https://github.com/arktos-ai-lab/felix-cat-classic).

## Support Arktos AI Lab

Arktos AI Lab is a one-person operation run by Ernst van Gassen — an independent researcher with one too many interests and not enough hours in the day. Building a translation tool from scratch in Rust, while also preserving the original Felix archive and keeping Felix Classic working on modern Windows, is genuinely a lot of side-project energy.

If Cheshire CAT has been useful to you, or if you just appreciate the work that goes into keeping open-source translation tooling alive — a small donation is a meaningful way to support the journey.

[![Donate via PayPal](https://img.shields.io/badge/Donate%20via-PayPal-blue.svg)](https://paypal.me/VanGassen)

Thank you. It genuinely makes a difference.
