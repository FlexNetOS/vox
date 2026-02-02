# vox

TTS CLI for macOS — local voice synthesis with Qwen and system `say`.

## Features

- **Two backends**: macOS native `say` and [Qwen TTS](https://github.com/ml-explore/mlx-audio) (local, Apple Silicon)
- **Voice cloning**: clone a voice from an audio sample, use it for all speech
- **Voice chat**: have a spoken conversation with Claude (STT + LLM + TTS)
- **Pipeline playback**: multi-sentence text plays without gaps between chunks
- **Preferences**: persist backend, voice, language, rate, style settings

## Install

```bash
# Quick install (macOS)
curl -fsSL https://raw.githubusercontent.com/rtk-ai/vox/main/install.sh | sh

# From source
cargo install --path .

# Or via Homebrew (coming soon)
brew tap rtk-ai/tap && brew install vox
```

### Requirements

- macOS (uses `say` and `afplay`)

For the Qwen backend (local neural TTS, Apple Silicon only):

```bash
brew install python3
pip install mlx-audio
```

This pulls in [mlx-audio](https://github.com/ml-explore/mlx-audio) which provides both TTS (`mlx_audio.tts`) and STT (`mlx_audio.stt`). The model `mlx-community/Qwen3-TTS-12Hz-0.6B-Base-bf16` is downloaded automatically on first use (~1.2 GB).

For voice chat:

```bash
brew install sox                  # audio recording (rec command)
export ANTHROPIC_API_KEY=sk-ant-...
```

## Usage

```bash
# Speak text (default: say backend)
vox "Hello, world."

# Use Qwen backend
vox -b qwen "Bonjour le monde."

# Pipe from stdin
echo "Hello" | vox

# List voices
vox --list-voices
vox -b qwen --list-voices

# Set voice and language
vox -b qwen -v Chelsie -l en "Good morning."
```

### Voice cloning

```bash
# Add a clone from an audio file
vox clone add patrick --audio ~/voice.wav --text "Transcription of the audio"

# Record a clone from microphone
vox clone record myvoice --duration 10

# Use a cloned voice
vox -v patrick "This speaks with your voice."

# List / remove clones
vox clone list
vox clone remove patrick
```

### Voice chat

```bash
# Start a voice conversation with Claude
export ANTHROPIC_API_KEY=sk-ant-...
vox chat
vox chat -v patrick -l fr
```

### Preferences

```bash
vox config show
vox config set backend qwen
vox config set lang fr
vox config set voice Chelsie
vox config reset
```

### Stats

```bash
vox stats
```

## AI Integration

Set up your project so that Claude Code provides spoken summaries after completing tasks:

```bash
cd your-project
vox init
```

This creates:
- **CLAUDE.md** — instructions for Claude to call `vox` after significant tasks
- **.claude/settings.json** — a `Stop` hook that says "Terminé" after each response

Running `vox init` again is safe — it skips files that are already configured.

## Configuration

| Env var | Description |
|---------|-------------|
| `VOX_CONFIG_DIR` | Override config directory (default: `~/.config/vox/`) |
| `VOX_DB_PATH` | Override database path (default: `~/.config/vox/vox.db`) |
| `ANTHROPIC_API_KEY` | Required for `vox chat` |

## License

[MIT](LICENSE)
