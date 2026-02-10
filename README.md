# vox

Cross-platform TTS CLI — local voice synthesis with three backends.

```
                         vox
                          |
            +-------------+-------------+
            |             |             |
          say          qwen        qwen-native
       (macOS)     (MLX/Python)    (pure Rust)
        native      Apple Silicon   cross-platform
                                   CPU/Metal/CUDA
                          |
                        rodio
                    (audio playback)
```

## Install

```bash
# From source
cargo install --path .

# Quick install (macOS / Linux / WSL)
curl -fsSL https://raw.githubusercontent.com/rtk-ai/vox/main/install.sh | sh
```

| Platform | Default backend | GPU |
|----------|----------------|-----|
| macOS | `say` | `--features metal` |
| Linux / WSL | `qwen-native` | `--features cuda` |

Linux requires `sudo apt install libasound2-dev`.

## Usage with Claude Code

```bash
vox init                # all integrations (default)
vox init -m mcp         # MCP server only
vox init -m cli         # CLI hook only
vox init -m skill       # slash command only
```

Each mode sets up a different integration:

| Mode | What it does |
|------|-------------|
| `mcp` | Registers `vox serve` as an MCP server in `~/.claude.json` (Claude Code) and Claude Desktop config. Exposes 8 tools: `vox_speak`, `vox_list_voices`, `vox_clone_*`, `vox_config_*`, `vox_stats`. |
| `cli` | Creates a `CLAUDE.md` in your project with instructions for Claude to call `vox` after significant tasks. Adds a `Stop` hook in `.claude/settings.json` that says "Terminé" after each response. |
| `skill` | Creates a `/speak` slash command in `~/.claude/commands/speak.md`. |
| `all` | Runs all three modes (default). |

```
  Claude Code
      |
   MCP stdio
      |
  vox serve ──> vox_speak, vox_list_voices, ...
```

Running `vox init` again is safe — it skips files that are already configured.

## Standalone CLI

```bash
vox "Hello, world."
vox -b qwen-native "Cross-platform TTS."
echo "Hello" | vox
vox --list-voices
```

### Voice cloning

```bash
vox clone add patrick --audio ~/voice.wav --text "Transcription"
vox clone record myvoice --duration 10
vox -v patrick "This speaks with your voice."
vox clone list
vox clone remove patrick
```

### Preferences

```bash
vox config show
vox config set backend qwen
vox config set lang fr
vox config set voice Chelsie
vox config reset
```

### Optional: Qwen backend (macOS)

Neural TTS via Python/MLX on Apple Silicon:

```bash
uv pip install mlx-audio
```

Model downloaded automatically on first use (~1.2 GB).

## Data

All state is stored locally in `~/.config/vox/`:

```
~/.config/vox/
├── vox.db          # SQLite: preferences, voice clones, usage logs
└── clones/         # audio files for voice clones
```

| Env var | Description |
|---------|-------------|
| `VOX_CONFIG_DIR` | Override config directory |
| `VOX_DB_PATH` | Override database path |

## License

[MIT](LICENSE)
