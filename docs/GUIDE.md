# Guide utilisateur

## Installation

### Script rapide (macOS / Linux / WSL)

```bash
curl -fsSL https://raw.githubusercontent.com/rtk-ai/vox/main/install.sh | sh
```

### Depuis les sources

```bash
# Standard (CPU)
cargo install --path .

# macOS avec GPU Metal (recommande sur Apple Silicon)
cargo install --path . --features metal

# Linux avec GPU NVIDIA
cargo install --path . --features cuda
```

### Prerequis optionnels

| Composant | Pour quoi | Installation |
|-----------|----------|--------------|
| `mlx-audio` | Backend qwen (macOS) | `uv pip install mlx-audio` |
| `sox` | Enregistrement micro (clone record, hear) | `brew install sox` / `apt install sox` |

## Demarrage rapide

```bash
# Parler
vox "Hello, world!"

# En francais
vox -l fr "Bonjour le monde"

# Pipe depuis une commande
echo "Message important" | vox

# Voir les voix
vox --list-voices
```

## Configuration avec un assistant IA

La methode recommandee est d'utiliser vox comme serveur MCP :

```bash
vox init
```

Cette commande configure automatiquement **tous les outils IA** presents sur votre machine (Claude Code, Cursor, VS Code, Zed, etc.). Redemarrez votre outil apres l'init.

L'assistant IA pourra alors :
- Vous parler apres avoir termine une tache
- Lister et changer les voix
- Gerer vos voice clones
- Jouer des sons de packs
- Ecouter votre voix (STT, macOS)

### Exemple avec Claude Code

Apres `vox init`, Claude Code peut utiliser les outils MCP directement :

```
> Corrige le bug dans auth.rs

[Claude corrige le bug, puis parle :]
"Le bug d'authentification a ete corrige. Le token etait expire
 car la duree etait en secondes au lieu de millisecondes."
```

## Personnaliser la voix

```bash
# Changer le backend par defaut
vox config set backend kokoro

# Voix feminine, style chaleureux
vox config set gender feminine
vox config set style warm

# Langue par defaut
vox config set lang fr

# Voir la config actuelle
vox config show
```

## Cloner votre voix

Creez un clone vocal a partir d'un enregistrement audio :

```bash
# Depuis un fichier existant
vox clone add mavoix --audio ~/enregistrement.wav --text "Transcription exacte"

# Enregistrer depuis le micro (necessite sox)
vox clone record mavoix --duration 10 --text "Ce que je dis"

# Utiliser le clone
vox -v mavoix "Ceci parle avec ma voix clonee"
```

Pour de meilleurs resultats :
- Enregistrement de 5-15 secondes
- Environnement calme, sans bruit de fond
- Parler naturellement, pas trop vite
- Fournir la transcription exacte avec `--text`

## Sound packs

Ajoutez des sons thematiques (style Warcraft peon, StarCraft, etc.) :

```bash
vox pack install peon          # Installer
vox pack set peon              # Activer
vox pack play greeting         # "Ready to work!"
vox pack play complete         # "Work complete."
vox pack play error            # "Can't do that."
```

Voir les packs disponibles : `vox pack list`

## Conversation vocale (macOS)

Discutez vocalement avec Claude :

```bash
export ANTHROPIC_API_KEY=sk-ant-...
vox chat -l fr
```

La boucle : vous parlez → Whisper transcrit → Claude repond → vox parle la reponse.

## Transcription (macOS)

Enregistrez et transcrivez votre voix :

```bash
vox hear -l fr
# Parlez... (s'arrete apres 2s de silence)
# => "Votre texte transcrit ici"
```

## Backends en detail

### say (macOS natif)
- Latence quasi-nulle (~100ms)
- Voix Apple integrees (Samantha, Thomas, etc.)
- Support du debit (`-r 200`)
- Pas de voice cloning

### kokoro (Rust pur)
- Cross-platform, zero dependance externe
- Bonne qualite vocale
- ~2-5s de latence
- Pas de voice cloning

### qwen (MLX Python, macOS)
- Qualite neurale superieure
- Voice cloning supporte
- Necessite `mlx-audio` + Apple Silicon
- ~1s warm / ~5-15s cold start

### qwen-native (Rust pur)
- Meme modele Qwen3-TTS mais en Rust (candle)
- Voice cloning supporte
- GPU via Metal (macOS) ou CUDA (Linux)
- Cross-platform

## Depannage

### "command not found: vox"
Le binaire n'est pas dans votre PATH. Verifiez avec `which vox` ou reinstallez.

### "Backend 'say' is only available on macOS"
Utilisez `kokoro` ou `qwen-native` sur Linux/Windows : `vox config set backend kokoro`

### Backend qwen lent au premier lancement
Normal : le modele (~1.2 GB) est telecharge automatiquement. Les lancements suivants sont plus rapides.

### Enregistrement micro ne fonctionne pas
Installez sox : `brew install sox` (macOS) ou `apt install sox` (Linux).

### "ANTHROPIC_API_KEY is required" (chat mode)
Exportez votre cle API : `export ANTHROPIC_API_KEY=sk-ant-...`
