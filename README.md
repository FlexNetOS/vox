# vox

Cross-platform TTS CLI with five backends and MCP server for AI assistants.

```
                              vox
                               |
       +--------+--------+----+----+--------+-----------+
       |        |        |         |        |           |
     say      qwen    qwen-native kokoro  voxtream    (TUI)
   (macOS)  (MLX/Py)  (Rust/candle) (ONNX) (zero-shot) vox setup
   native   Apple Si.  CPU/Metal  CPU/GPU  CUDA/MPS
                        /CUDA
                          |
                        rodio (audio playback)
```

## Backends

| Backend | Engine | Voice cloning | Latency (cold) | Latency (warm) | GPU | Platform |
|---------|--------|:---:|---:|---:|:---:|----------|
| `say` | macOS native | No | **3s** | **3s** | No | macOS |
| `kokoro` | ONNX via Python | No | **10s** | **10s** | No | All |
| `qwen-native` | Candle (Rust) | Yes | **11m33s** | ~3s | Metal/CUDA | All |
| `voxtream` | PyTorch 0.5B | Yes | **68s** | ~8s | CUDA/MPS | All |
| `qwen` | MLX-Audio (Python) | Yes | ~15s | ~2s | Apple Neural | macOS |

### Benchmark — single sentence (~50 chars)

Real-world measurements. Cold start = first run (includes model loading). Warm = model cached on disk.

| Backend | M2 Pro (CPU) | RTX 4070 Ti SUPER (CUDA) | Voice cloning | Quality |
|---------|-------------:|-------------------------:|:---:|---------|
| **`say`** | **3s** | macOS only | No | System voices |
| **`kokoro`** | **10s** | ~10s | No | Good |
| **`voxtream`** | **68s** / 8s warm | **44s** / **22s** warm | Yes (zero-shot) | Excellent |
| **`qwen-native`** | **11m33s** / 3s warm | ~30s / ~2s warm | Yes | Excellent |
| **`qwen`** | ~15s / 2s warm | macOS only | Yes | Excellent |

> `voxtream` cold start includes model download (~500MB) on first run. Subsequent "warm" runs reuse cached model.
> `qwen-native` benefits massively from `--features metal` (macOS) or `--features cuda` (Linux).
> For lowest latency: `say` (macOS) or `kokoro` (all platforms). For best quality + cloning: `voxtream` on GPU.

## Install

```bash
# Quick install (macOS / Linux / WSL)
curl -fsSL https://raw.githubusercontent.com/rtk-ai/vox/main/install.sh | sh

# From source
cargo install --path .

# With GPU acceleration
cargo install --path . --features metal  # macOS Apple Silicon
cargo install --path . --features cuda   # Linux NVIDIA
```

### VoXtream backend (optional)

```bash
brew install espeak-ng                              # macOS (or apt install espeak-ng on Linux)
uv venv ~/.local/venvs/voxtream --python 3.11
uv pip install --python ~/.local/venvs/voxtream/bin/python "voxtream>=0.2"
# Copy config files
git clone --depth 1 https://github.com/herimor/voxtream.git /tmp/voxtream-repo
cp /tmp/voxtream-repo/configs/*.json "$(vox config show 2>/dev/null | head -1 | grep -v backend || echo ~/.config/vox)/voxtream/"
```

| Platform | Default backend | GPU |
|----------|----------------|-----|
| macOS | `say` | `--features metal` |
| Linux / WSL | `kokoro` | `--features cuda` |

Linux requires `sudo apt install libasound2-dev`.

## Quick start

```bash
vox "Hello, world."                     # Speak with default backend
vox -b voxtream "Zero-shot TTS."        # VoXtream2 (fastest neural)
vox -b kokoro -l fr "Bonjour"           # Kokoro with language
echo "Piped text" | vox                 # Read from stdin
vox --list-voices                       # List available voices
vox setup                               # Interactive TUI configuration
```

## Interactive setup (TUI)

For humans — choose backend, voice, language, and style interactively:

```bash
vox setup
```

```
┌ Backend ──┐┌ Voice ─────┐┌ Lang ┐┌ Style ────┐┌ Config ──────┐
│> say      ││> Samantha  ││> en  ││> (default)││ Backend: say │
│  kokoro   ││  Thomas    ││  fr  ││  calm     ││ Voice: ...   │
│  qwen-nat ││  Amelie    ││  es  ││  warm     ││ Lang:  en    │
│  voxtream ││           ││  de  ││  cheerful ││              │
│  qwen     ││           ││  ja  ││          ││ [T]est [S]ave│
└───────────┘└────────────┘└──────┘└──────────┘└──────────────┘
```

Navigate with arrow keys / hjkl, Tab to switch panel, T to test, S to save, Q to quit.

AI agents use CLI flags instead: `vox -b voxtream -l fr "text"`

## AI assistant integration

One command configures **14 AI tools** (Claude Code, Cursor, VS Code, Zed, Codex, Gemini, Amazon Q, and more):

```bash
vox init                # MCP server (default) — all AI tools
vox init -m cli         # CLAUDE.md + Stop hook (recommended)
vox init -m skill       # /speak slash command
vox init -m all         # all of the above
```

Running `vox init` again is safe — it skips files that are already configured.

### CLI mode vs MCP mode

**CLI mode is recommended** for AI coding agents. Benchmarks show CLI tools are [10-32x cheaper and 100% reliable vs 72% for MCP](https://mariozechner.at/posts/2025-08-15-mcp-vs-cli/) due to MCP's TCP timeout overhead and JSON schema cost per call.

| Mode | Reliability | Token cost | Best for |
|------|------------|------------|----------|
| **CLI** (`vox init -m cli`) | 100% | Low (Bash call) | Claude Code, Codex, terminal agents |
| **MCP** (`vox init`) | ~72% | Higher (JSON schema) | Cursor, VS Code, GUI-based tools |

## Voice cloning

```bash
vox clone add patrick --audio ~/voice.wav --text "Transcription"
vox clone record myvoice --duration 10
vox -v patrick "This speaks with your voice."
vox clone list
vox clone remove patrick
```

Works with `qwen`, `qwen-native`, and `voxtream` backends. VoXtream2 uses zero-shot cloning (3-10s audio prompt, no training needed).

## Preferences

```bash
vox config show
vox config set backend voxtream
vox config set lang fr
vox config set voice Chelsie
vox config set gender feminine
vox config set style warm
vox config reset
```

## Sound packs

```bash
vox pack install peon              # Install a pack
vox pack set peon                  # Activate it
vox pack play greeting             # Play a sound
vox pack list                      # List available packs
```

## Voice conversation (macOS)

```bash
export ANTHROPIC_API_KEY=sk-...
vox chat -l fr                     # Talk with Claude
vox hear -l fr                     # Speech-to-text only
```

## Data

All state is stored locally — no data sent to external servers (except `vox chat` which uses Claude API).

```
~/.config/vox/           # or ~/Library/Application Support/vox/ on macOS
  vox.db                 # SQLite: preferences, voice clones, usage logs
  clones/                # Audio files for voice clones
  packs/                 # Installed sound packs
  voxtream/              # VoXtream2 config files
```

| Env var | Description |
|---------|-------------|
| `VOX_CONFIG_DIR` | Override config directory |
| `VOX_DB_PATH` | Override database path |

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | Technical architecture, backends, DB schema, MCP protocol, security |
| [Features](docs/FEATURES.md) | All commands and features documented |
| [Guide](docs/GUIDE.md) | Installation, quick start, troubleshooting |

## License

[Source-Available](LICENSE) — Free for individuals and teams up to 20 people. Enterprise license required for larger organizations. Contact: license@rtk.ai
