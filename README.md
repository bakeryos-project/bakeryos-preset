# BakeryOS Preset Configuration Guide

This guide explains how to write and structure a preset configuration file (`.yaml`) for BakeryOS using the standard `Example preset` template.

## The Example Preset File

```yaml
name: Example preset
id: bakeryos.presets.Example
stages:
  - name: Hello
    triggers: ["apply"]
    packages:
      - name: fastfetch
    backups:
      - "/etc/issue"

  - name: Restore
    triggers: ["apply"]
    packages:
      - name: fastfetch
    restores:
      - "/etc/issue"

```

---

## Structure Breakdown

A BakeryOS preset consists of global metadata and an ordered list of execution blocks called **Stages**.

### 1. Global Metadata

* **`name`**: A human-readable display name for the preset.
* **`id`**: A unique dot-separated identifier (e.g., `bakeryos.presets.Example`) used by the system storage `HashMap` to index configuration paths and manage duplicate checks.

### 2. Stage Properties

Each item under the `stages` list represents an isolated execution phase. A stage contains the following attributes:

| Field | Type | Description |
| --- | --- | --- |
| `name` | String | The designation of the stage, displayed during execution (e.g., `Stage #Hello`). |
| `triggers` | List of Strings | The action flags that activate this stage (e.g., `"apply"`, `"unapply"`, or `"rollback"`). |
| `packages` | List of Objects | System packages that need to be managed or verified via `pacman` during this stage. |
| `backups` | List of Strings | Absolute paths to system configuration files that must be safely backed up before modification. |
| `restores` | List of Strings | Absolute paths to system files that need to be reverted using their tracked backup UUIDs. |

---

## How Lifecycle Actions Work

BakeryOS parses the stages sequentially from top to bottom and filters them based on the `with_trigger` arguments passed to the runtime executor.

### The Backup Pipeline (`backups`)

When a stage containing `backups` is triggered:

1. The engine checks if the system file exists.
2. A unique `UUID v4` is generated to create a hashed backup file (e.g., `bakeryos.presets.Example+uuid.backup`).
3. The mapping is stored in the central `index.json` registry via a `HashMap` lookup table.

### The Restoration Pipeline (`restores`)

When a stage containing `restores` is triggered:

1. The engine queries the internal database for the corresponding original file path.
2. It fetches the backup file name from the database index.
3. The cached backup content is copied back to overwrite the current target file, effectively performing an **unapply** / **rollback** operation.

---

## Best Practices for Production

While the example preset assigns the `["apply"]` trigger to both stages for rapid testing loop cycles, actual production presets should segregate operations to prevent immediate rollbacks:

* Set setup actions to `triggers: ["apply"]`.
* Set cleanup/reversion actions to `triggers: ["unapply"]` or `triggers: ["rollback"]`.

