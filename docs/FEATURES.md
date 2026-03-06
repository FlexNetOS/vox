# Documentation fonctionnelle

## Synthese vocale (TTS)

Fonctionnalite principale : transformer du texte en parole.

```bash
vox "Bonjour le monde"              # Backend par defaut
vox -b kokoro "Hello world"         # Backend specifique
vox -b qwen-native -l fr "Salut"   # Backend + langue
echo "Texte pipe" | vox             # Lecture depuis stdin
```

### Options de parole

| Flag | Description | Exemple |
|------|-------------|---------|
| `-b` | Backend TTS | `-b kokoro`, `-b say`, `-b qwen-native` |
| `-v` | Voix ou clone | `-v Chelsie`, `-v patrick` |
| `-l` | Langue | `-l fr`, `-l ja`, `-l en` |
| `-r` | Debit (mots/min, backend say) | `-r 200` |
| `--gender` | Genre vocal | `--gender feminine` |
| `--style` | Intonation | `--style warm`, `--style energetic` |
| `-m` | Modele TTS | `-m mlx-community/Qwen3-TTS-12Hz-0.6B-Base-4bit` |

### Langues supportees

en, fr, es, de, it, pt, zh, ja, ko, ru, ar, nl

### Styles d'intonation

calm, energetic, warm, authoritative, cheerful, serious

## Voice cloning

Cloner une voix a partir d'un fichier audio de reference.

```bash
# Ajouter un clone depuis un fichier
vox clone add patrick --audio ~/voice.wav --text "Transcription du fichier"

# Enregistrer directement depuis le micro (necessite sox)
vox clone record myvoice --duration 10 --text "Ce que je dis pendant l'enregistrement"

# Utiliser un clone
vox -v patrick "Ceci parle avec ma voix"

# Gerer les clones
vox clone list
vox clone remove patrick
```

Formats audio acceptes : wav, mp3, flac, ogg, m4a.

Le voice cloning bascule automatiquement sur le backend `qwen` (macOS) ou `qwen-native` (autres) car `say` et `kokoro` ne supportent pas le cloning.

## Configuration

Preferences persistantes en base SQLite.

```bash
vox config show                    # Afficher les preferences
vox config set backend kokoro      # Changer le backend par defaut
vox config set lang fr             # Langue par defaut
vox config set voice Chelsie       # Voix par defaut
vox config set gender feminine     # Genre vocal
vox config set style warm          # Style d'intonation
vox config set rate 180            # Debit (say uniquement)
vox config set model <model_id>    # Modele TTS specifique
vox config reset                   # Reinitialiser tout
```

Priorite de resolution : **flags CLI > preferences DB > valeurs par defaut**.

## Sound packs

Packs de sons thematiques (compatible peon-ping). Sons courts joues pour signaler des evenements.

```bash
vox pack list                      # Voir packs installes + disponibles
vox pack install peon              # Installer un pack
vox pack set peon                  # Activer un pack
vox pack play greeting             # Jouer un son de la categorie "greeting"
vox pack play error -p peon_fr     # Jouer depuis un pack specifique
vox pack remove peon               # Desinstaller un pack
```

Categories de sons : greeting, acknowledge, complete, error, permission, annoyed.

## Statistiques d'utilisation

Dashboard complet de l'historique TTS.

```bash
vox stats
```

Affiche :
- Temps de parole total (format humain : h/m/s)
- Nombre total d'appels et de caracteres
- Latence moyenne et throughput (chars/s)
- Repartition par backend (calls, chars, duree, moyenne)
- Repartition par langue (avec barres visuelles)
- 10 derniers appels avec details

## Integration IA (`vox init`)

Configuration automatique pour 14 outils IA en une commande.

```bash
vox init                # Mode MCP (defaut) — configure tous les outils
vox init -m cli         # Mode CLI — CLAUDE.md + Stop hook
vox init -m skill       # Mode Skill — commande /speak
vox init -m all         # Les trois modes
```

### Outils supportes (mode MCP)

| Outil | Config |
|-------|--------|
| Claude Code | `~/.claude.json` |
| Claude Desktop | Config specifique OS |
| Cursor | `~/.cursor/mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` |
| VS Code / Copilot | `Code/User/mcp.json` |
| Zed | `~/.config/zed/settings.json` |
| Codex | `~/.codex/config.toml` |
| OpenCode | `~/.config/opencode/opencode.json` |
| Gemini Code Assist | `~/.gemini/settings.json` |
| Amazon Q | `~/.aws/amazonq/mcp.json` |
| Cline | Extension VS Code globalStorage |
| Roo Code | Extension VS Code globalStorage |
| Kilo Code | Extension VS Code globalStorage |
| Amp | `~/.ampcode/settings.json` |

L'init est idempotent : relancer `vox init` ne duplique pas les configurations.

### Mode CLI

Cree un `CLAUDE.md` dans le projet courant avec des instructions pour que l'assistant appelle `vox` apres les taches significatives. Ajoute un hook `Stop` dans `.claude/settings.json` qui dit "Termine" a la fin de chaque reponse.

### Mode Skill

Cree une commande `/speak` dans `~/.claude/commands/speak.md` pour invoquer vox via slash command.

## Speech-to-Text (macOS)

Transcription locale via mlx-whisper.

```bash
# CLI
vox hear -l fr                     # Ecoute + transcription en francais
vox hear -l en -t 60 -s 3.0       # Timeout 60s, silence 3s

# Via MCP
vox_hear                           # Utilise par l'assistant IA
```

Prerequis : `sox` (pour l'enregistrement micro) et `mlx-audio` (pour la transcription).

## Mode conversation (macOS)

Boucle vocale complete : ecouter → reflechir → parler.

```bash
export ANTHROPIC_API_KEY=sk-...
vox chat                           # Conversation avec Claude
vox chat -v patrick -l fr          # Avec voice clone en francais
```

Utilise Claude API en streaming pour la reflexion, STT local pour l'ecoute, et TTS pour la reponse.

## Lecture de voix

Lister les voix disponibles pour un backend.

```bash
vox --list-voices                  # Backend par defaut
vox -b say --list-voices           # Voix macOS
vox -b kokoro --list-voices        # Voix Kokoro
```

## Donnees locales

Tout est stocke localement, aucune donnee n'est envoyee a un serveur externe (sauf le mode chat qui utilise l'API Claude).

```
~/.config/vox/
  vox.db          # SQLite : preferences, clones, logs
  clones/         # Fichiers audio des voice clones
  packs/          # Sound packs installes
```

Variables d'environnement :
- `VOX_CONFIG_DIR` — repertoire de configuration alternatif
- `VOX_DB_PATH` — chemin de base de donnees alternatif
