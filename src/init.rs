use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

const VOX_START: &str = "<!-- vox:start -->";
const VOX_END: &str = "<!-- vox:end -->";

pub struct InitResult {
    pub claude_md_written: bool,
    pub settings_written: bool,
}

/// Returns the full markdown block for a new CLAUDE.md.
pub fn claude_md_block() -> String {
    format!(
        r#"{VOX_START}
## Voice feedback (vox)

This project uses [vox](https://github.com/rtk-ai/vox) for spoken feedback.

After completing a significant task (feature, bug fix, refactor), provide an oral summary:

```bash
vox -b say "Short summary of what was done, in French"
```

Keep summaries under 2 sentences. Use French.
Do not speak for trivial operations (formatting, single-line fixes).
{VOX_END}"#
    )
}

/// Returns a short block to append to an existing CLAUDE.md.
pub fn claude_md_append_block() -> String {
    format!(
        r#"{VOX_START}
After completing a significant task, call `vox -b say "résumé en français"` to give a spoken summary.
{VOX_END}"#
    )
}

/// Checks whether the CLAUDE.md content already contains vox markers.
pub fn claude_md_has_vox(content: &str) -> bool {
    content.contains(VOX_START)
}

/// Checks whether a parsed settings.json already has a vox hook.
pub fn has_vox_hook(settings: &Value) -> bool {
    if let Some(hooks) = settings.get("hooks") {
        if let Some(stop) = hooks.get("Stop") {
            if let Some(arr) = stop.as_array() {
                for entry in arr {
                    if let Some(inner_hooks) = entry.get("hooks") {
                        if let Some(inner_arr) = inner_hooks.as_array() {
                            for h in inner_arr {
                                if let Some(cmd) = h.get("command").and_then(|c| c.as_str()) {
                                    if cmd.starts_with("vox ") {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Builds the settings.json content, merging with existing content if provided.
pub fn build_settings(existing: Option<&str>) -> Result<String> {
    let vox_hook: Value = serde_json::from_str(
        r#"{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "vox -b say \"Terminé.\""
          }
        ]
      }
    ]
  }
}"#,
    )?;

    let merged = match existing {
        Some(content) => {
            let mut base: Value =
                serde_json::from_str(content).context("Invalid JSON in settings.json")?;

            if has_vox_hook(&base) {
                return serde_json::to_string_pretty(&base).context("Failed to serialize settings");
            }

            // Merge hooks
            if let Some(new_hooks) = vox_hook.get("hooks") {
                if let Some(new_stop) = new_hooks.get("Stop") {
                    let base_obj = base
                        .as_object_mut()
                        .context("settings.json is not an object")?;
                    let hooks_obj = base_obj
                        .entry("hooks")
                        .or_insert_with(|| Value::Object(serde_json::Map::new()));
                    let hooks_map = hooks_obj
                        .as_object_mut()
                        .context("hooks is not an object")?;

                    let stop_arr = hooks_map
                        .entry("Stop")
                        .or_insert_with(|| Value::Array(vec![]));
                    if let Some(arr) = stop_arr.as_array_mut() {
                        if let Some(new_entries) = new_stop.as_array() {
                            arr.extend(new_entries.clone());
                        }
                    }
                }
            }

            base
        }
        None => vox_hook,
    };

    serde_json::to_string_pretty(&merged).context("Failed to serialize settings")
}

/// Orchestrates the full init: writes CLAUDE.md and .claude/settings.json.
pub fn run_init(project_dir: &Path) -> Result<InitResult> {
    let mut result = InitResult {
        claude_md_written: false,
        settings_written: false,
    };

    // --- CLAUDE.md ---
    let claude_md_path = project_dir.join("CLAUDE.md");
    if claude_md_path.exists() {
        let content = fs::read_to_string(&claude_md_path).context("Failed to read CLAUDE.md")?;
        if !claude_md_has_vox(&content) {
            let new_content = format!("{}\n\n{}\n", content.trim_end(), claude_md_append_block());
            fs::write(&claude_md_path, new_content).context("Failed to write CLAUDE.md")?;
            result.claude_md_written = true;
        }
    } else {
        fs::write(&claude_md_path, format!("{}\n", claude_md_block()))
            .context("Failed to create CLAUDE.md")?;
        result.claude_md_written = true;
    }

    // --- .claude/settings.json ---
    let claude_dir = project_dir.join(".claude");
    let settings_path = claude_dir.join("settings.json");

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path).context("Failed to read settings.json")?;
        let parsed: Value =
            serde_json::from_str(&content).context("Invalid JSON in settings.json")?;

        if !has_vox_hook(&parsed) {
            let new_content = build_settings(Some(&content))?;
            fs::write(&settings_path, format!("{}\n", new_content))
                .context("Failed to write settings.json")?;
            result.settings_written = true;
        }
    } else {
        fs::create_dir_all(&claude_dir).context("Failed to create .claude directory")?;
        let content = build_settings(None)?;
        fs::write(&settings_path, format!("{}\n", content))
            .context("Failed to create settings.json")?;
        result.settings_written = true;
    }

    Ok(result)
}
