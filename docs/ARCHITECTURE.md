# Architecture technique

## Vue d'ensemble

vox est un CLI TTS cross-platform ecrit en Rust. Il transforme du texte en parole via quatre backends interchangeables et expose ses fonctionnalites comme serveur MCP (Model Context Protocol) pour l'integration avec les assistants IA.

```
                          vox (Rust)
                              |
            +---------+-------+-------+---------+
            |         |               |         |
          say      qwen          qwen-native  kokoro
       (macOS)  (MLX/Python)    (pure Rust)  (pure Rust)
       native   Apple Silicon   CPU/Metal    CPU/GPU
                                /CUDA
                    |
                  rodio (audio playback cross-platform)
```

## Modules source

```
src/
  main.rs         CLI (clap) — parsing args, dispatch subcommands
  lib.rs          Exports publics des modules
  mcp.rs          Serveur MCP JSON-RPC stdio (14 tools)
  backend/
    mod.rs        Trait TtsBackend + dispatch get_backend()
    say.rs        Backend macOS natif (NSSSpeechSynthesizer via /usr/bin/say)
    qwen.rs       Backend MLX-Audio Python (Apple Silicon, macOS only)
    qwen_native.rs Backend candle/Rust (Qwen3-TTS, cross-platform)
    kokoro.rs     Backend Kokoro-TTS (pure Rust, cross-platform)
  config.rs       Chemins config, constantes, enums (Gender, IntonationStyle)
  db.rs           SQLite (rusqlite) — preferences, clones, usage logs, stats
  init.rs         Auto-configuration pour 14 outils IA
  input.rs        Lecture texte (args, stdin, pipe)
  clone.rs        Voice cloning — validation audio, enregistrement micro
  pack.rs         Sound packs (peon-ping compatible)
  audio.rs        Playback audio via rodio
  stt.rs          Speech-to-text via mlx-whisper (macOS only)
  chat/           Mode conversation vocale (macOS only)
```

## Backends TTS

| Backend | Plateforme | Dependance | Latence | Voice cloning |
|---------|-----------|------------|---------|---------------|
| `say` | macOS uniquement | Aucune (systeme) | ~100ms | Non |
| `qwen` | macOS (Apple Silicon) | `mlx-audio` (Python) | ~5-15s (cold) / ~1s (warm) | Oui |
| `qwen-native` | Toutes | Aucune (Rust pur) | ~3-10s | Oui |
| `kokoro` | Toutes | Aucune (Rust pur) | ~2-5s | Non |

### Trait TtsBackend

```rust
pub trait TtsBackend {
    fn name(&self) -> &str;
    fn speak(&self, text: &str, opts: &SpeakOptions) -> Result<()>;
    fn list_voices(&self) -> Result<Vec<String>>;
    fn is_available(&self) -> bool;
}
```

Chaque backend implemente ce trait. Le dispatch se fait via `get_backend(name)` dans `backend/mod.rs`.

### Defaut par plateforme

- **macOS**: `say` (zero latence, voix systeme)
- **Linux / Windows**: `kokoro` (Rust pur, pas de dependance Python)

## Base de donnees

SQLite via `rusqlite` avec WAL mode. Fichier: `~/.config/vox/vox.db`.

### Schema

```sql
-- Preferences utilisateur (une seule ligne, UPSERT)
CREATE TABLE preferences (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    backend TEXT, voice TEXT, lang TEXT, rate INTEGER,
    gender TEXT, style TEXT, model TEXT, pack TEXT
);

-- Voice clones
CREATE TABLE voice_clones (
    name TEXT PRIMARY KEY,
    ref_audio TEXT NOT NULL,
    ref_text TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Journal d'utilisation
CREATE TABLE usage_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT DEFAULT (datetime('now')),
    backend TEXT NOT NULL,
    voice TEXT, lang TEXT,
    text_len INTEGER NOT NULL,
    duration_ms INTEGER
);
```

### Requetes d'agregation

- `get_usage_summary()` — total calls + total chars
- `get_backend_stats()` — calls, chars, duration par backend
- `get_lang_stats()` — calls par langue
- `get_total_duration_ms()` — temps de parole cumule
- `get_usage_stats()` — 50 dernieres entrees

## Protocole MCP

Serveur JSON-RPC 2.0 sur stdio. Compatible MCP spec `2024-11-05`.

### Lifecycle

1. Client envoie `initialize` → serveur repond avec capabilities + tools
2. Client envoie `initialized` (notification)
3. Client appelle `tools/call` avec `name` et `arguments`
4. Serveur repond avec `content[{type: "text", text: "..."}]`

### 14 outils MCP exposes

| Tool | Description |
|------|-------------|
| `vox_speak` | Synthetise et joue du texte (params: text, voice, lang, backend) |
| `vox_list_voices` | Liste les voix disponibles pour un backend |
| `vox_clone_list` | Liste les voice clones enregistres |
| `vox_clone_add` | Ajoute un voice clone (name, audio_path, ref_text) |
| `vox_clone_remove` | Supprime un voice clone |
| `vox_config_show` | Affiche les preferences courantes |
| `vox_config_set` | Modifie une preference (key, value) |
| `vox_stats` | Statistiques d'utilisation |
| `vox_pack_list` | Liste les sound packs installes/disponibles |
| `vox_pack_install` | Installe un sound pack |
| `vox_pack_set` | Active un sound pack |
| `vox_pack_play` | Joue un son d'un pack (category) |
| `vox_pack_remove` | Supprime un sound pack |
| `vox_hear` | Enregistre et transcrit (STT, macOS only) |

## Compilation conditionnelle

```rust
#[cfg(target_os = "macos")]   // say, qwen, stt, chat
#[cfg(not(target_os = "macos"))] // kokoro comme defaut
```

Feature flags Cargo:
- `metal` — GPU Apple Silicon (Metal + Accelerate) pour qwen-native
- `cuda` — GPU NVIDIA pour qwen-native

## Securite

- **SQL injection**: Toutes les requetes utilisent des parametres lies (`?`). Les cles de preference sont validees par whitelist.
- **Path traversal**: Extensions audio validees (wav, mp3, flac, ogg, m4a). Fichiers verifies existants.
- **Input validation**: Backends, langues, gender, style valides par enum/whitelist.
- **Pas de shell**: Les commandes externes utilisent `std::process::Command` (pas de `sh -c`).

## CI/CD

- GitHub Actions: matrix macOS / Ubuntu / Windows
- release-please: `feat:` → version bump PR → merge → release + binaires
- Binaires: aarch64-apple-darwin, x86_64-apple-darwin (metal), x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc
- Tests: `cargo test` (unitaires + integration UX/security/perf)
