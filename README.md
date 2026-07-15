# GPUI Markdown Editor

**English** | [简体中文](README.zh-CN.md)

> [!IMPORTANT]
> ## Origin and attribution
> This project was created by extracting and adapting the Markdown editing core of [Velotype](https://github.com/manyougz/velotype), a native Rust + GPUI Markdown editor created by `manyougz`. The original project and its authors deserve credit for the editor architecture and implementation on which this component is based.
>
> This repository turns that editor core into an independent, reusable GPUI component. It is not a replacement for Velotype's complete application shell. The original Apache License 2.0 and copyright notice are retained in [LICENSE-APACHE](LICENSE-APACHE).

## Overview

`gpui-markdown-editor` is a native Markdown editor component written in Rust with [GPUI](https://gpui.rs/). It provides two editing representations over the same Markdown document:

- **Rendered mode** — block-based, WYSIWYG-style editing while preserving Markdown semantics.
- **Source mode** — raw Markdown editing in a single source buffer, with optional line numbers.

Switching modes serializes through canonical Markdown, rebuilds the target representation, and restores the selection by source offsets. Unsupported or unsafe structures fall back to raw Markdown blocks so they can remain visible, editable, and round-trip without silent data loss.

## Features

- Rendered and source editing modes.
- Paragraphs, headings, lists, task lists, block quotes, callouts, and thematic breaks.
- Inline emphasis, links, reference links, code, inline HTML, and math projections.
- Native table editing and table-axis operations.
- Fenced code blocks with optional Tree-sitter highlighting.
- Images, HTML blocks, LaTeX, footnotes, and Mermaid diagrams.
- UTF-8/UTF-16 selection mapping for IME, CJK text, and emoji.
- Document-level undo and redo, including cross-block edits.
- Source-offset selection API that remains stable across mode changes.
- Per-editor theme, strings, display settings, base directory, and image-paste policy.
- Host-facing events for changes, mode changes, selection, link requests, and errors.

## Component boundary

The crate owns Markdown editing state and rendering. The host application remains responsible for application-level behavior:

| Component owns | Host owns |
| --- | --- |
| Markdown document model | Open, save, and Save As flows |
| Rendered/source mode conversion | Dirty-state and close confirmation |
| Editing, selection, undo/redo | Window and workspace management |
| Markdown/block rendering | Menus and application preferences |
| Link activation requests | Actually opening external links |
| Image insertion transaction | Storage policy for encoded pasted images |
| Theme and string consumption | Loading and selecting themes/languages |
| Error events | User-facing application dialogs |

This separation allows the editor to be embedded in an existing GPUI window without importing Velotype's window chrome, workspace, persistence, export, or update systems.

## Repository layout

```text
.
├── Cargo.toml                         # Workspace and demo application
├── src/main.rs                        # Runnable GPUI demo
└── crates/gpui-markdown-editor/
    ├── Cargo.toml                     # Reusable component crate
    └── src/
        ├── lib.rs                     # Public facade
        ├── api.rs                     # Modes, commands, events, selection
        ├── environment.rs             # Per-instance configuration
        ├── host.rs                    # Image-paste host interface
        ├── editor/                    # Document/editor runtime
        └── components/                # Block and Markdown rendering runtime
```

## Requirements

- A current stable Rust toolchain with Rust 2024 edition support.
- GPUI `0.2.2` and its platform prerequisites.

Follow the [GPUI documentation](https://docs.rs/gpui/0.2.2/gpui/) for platform-specific setup.

## Run the demo

From the repository root:

```bash
cargo run -p gpui-markdown-editor-demo
```

The demo embeds the component in its own GPUI window. Use the **Rendered** and **Source** buttons, or `Ctrl/Cmd+Tab`, to switch modes.

## Use from another project

Add the component directly from GitHub:

```toml
[dependencies]
gpui = { version = "0.2.2", features = ["runtime_shaders"] }
gpui-markdown-editor = { git = "https://github.com/yuWorm/gpui-markdeditor.git", package = "gpui-markdown-editor" }
```

Cargo finds the `gpui-markdown-editor` package inside the remote workspace. Applications should commit `Cargo.lock` so builds keep using the resolved Git revision. A library or a project that requires an explicit revision can add `rev = "<commit-sha>"`.

For an SSH checkout, use:

```toml
gpui-markdown-editor = { git = "ssh://git@github.com/yuWorm/gpui-markdeditor.git", package = "gpui-markdown-editor" }
```

For local development, use a path dependency instead:

```toml
gpui-markdown-editor = { path = "../gpui-markdeditor/crates/gpui-markdown-editor" }
```

Change the path to the actual location of this repository.

## Basic integration

Register the component's default key bindings once during application startup:

```rust
use gpui::*;
use gpui_markdown_editor::default_key_bindings;

Application::new().run(|cx: &mut App| {
    cx.bind_keys(default_key_bindings());
    // Open the application's window here.
});
```

Create one editor entity in the host view and subscribe to its events:

```rust
use gpui::*;
use gpui_markdown_editor::{
    MarkdownEditor, MarkdownEditorEvent, MarkdownEditorOptions,
};

struct DocumentView {
    editor: Entity<MarkdownEditor>,
    dirty: bool,
}

impl DocumentView {
    fn new(markdown: String, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            MarkdownEditor::new(markdown, MarkdownEditorOptions::default(), cx)
        });
        cx.subscribe(&editor, Self::on_editor_event).detach();
        Self {
            editor,
            dirty: false,
        }
    }

    fn on_editor_event(
        &mut self,
        _editor: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            MarkdownEditorEvent::Changed { .. } => self.dirty = true,
            MarkdownEditorEvent::OpenLinkRequested(request) => {
                // Validate and open request.open_target in the host.
            }
            MarkdownEditorEvent::Error { message } => {
                // Present the error using the host application's UI.
            }
            MarkdownEditorEvent::ModeChanged { .. }
            | MarkdownEditorEvent::SelectionChanged(_) => {}
        }
        cx.notify();
    }
}

impl Render for DocumentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.editor.clone())
    }
}
```

See [`src/main.rs`](src/main.rs) for a complete window and toolbar example.

## Read and replace Markdown

```rust
// Read the current canonical Markdown when the host needs to save it.
let markdown = self.editor.read(cx).markdown(cx);

// Replace the document and reset its undo/redo history.
self.editor.update(cx, |editor, cx| {
    editor.replace_markdown(next_markdown, cx);
});
```

The `Changed` event intentionally carries only a revision number. This avoids allocating and serializing the entire document on every keystroke. Read `markdown()` only when the host actually needs the text, such as during save or export.

## Control the editing mode

```rust
use gpui_markdown_editor::{EditorCommand, MarkdownEditorMode};

self.editor.update(cx, |editor, cx| {
    editor.set_mode(MarkdownEditorMode::Source, cx);
    editor.execute(EditorCommand::Undo, cx);
    editor.execute(EditorCommand::Redo, cx);
    editor.toggle_mode(cx);
});
```

Available document commands are `Undo`, `Redo`, `ToggleMode`, and `SetMode`.

## Configure one editor instance

```rust
use std::{path::PathBuf, sync::Arc};
use gpui_markdown_editor::{
    MarkdownEditorEnvironment, MarkdownEditorOptions,
    MarkdownEditorStrings, MarkdownEditorTheme,
};

let environment = MarkdownEditorEnvironment {
    theme: Arc::new(MarkdownEditorTheme::default_theme()),
    strings: Arc::new(MarkdownEditorStrings::en_us()),
    document_base_dir: Some(PathBuf::from("/path/to/document")),
    show_source_line_numbers: true,
    show_table_headers: true,
    ..MarkdownEditorEnvironment::default()
};

let options = MarkdownEditorOptions {
    environment,
    history_limit: 200,
    ..MarkdownEditorOptions::default()
};
```

Use `set_environment` to replace these settings at runtime. Environment data is stored per editor instance; the component does not read Velotype application globals.

## Image paste policy

Local image paths are inserted directly by default. Encoded clipboard image bytes require the host to provide an implementation of `ImagePasteHandler`:

```rust
pub trait ImagePasteHandler: Send + Sync + 'static {
    fn materialize(
        &self,
        source: PastedImage,
        document_base_dir: Option<&Path>,
    ) -> anyhow::Result<ImageTarget>;
}
```

A host can use this boundary to copy files beside the document, write into an `assets` directory, upload to remote storage, or reject the operation. The method is synchronous so image insertion, selection updates, and undo capture remain one transaction.

## Cargo features

Default features:

- `code-highlight-core`
- `code-highlight-official`
- `code-highlight-config`
- `html-native`

To minimize parser dependencies, disable defaults and enable only the required capabilities:

```toml
gpui-markdown-editor = {
    path = "crates/gpui-markdown-editor",
    default-features = false,
    features = ["code-highlight-core", "html-native"]
}
```

## Testing

```bash
cargo check -p gpui-markdown-editor-demo
cargo test -p gpui-markdown-editor -- --test-threads=1
```

The component suite covers mode round-trips, Markdown import/serialization, block editing, selection mapping, tables, IME behavior, undo/redo, and per-instance configuration.

## License

Apache License 2.0. See [LICENSE-APACHE](LICENSE-APACHE).

This project retains the original Velotype copyright notice and license. Original project: <https://github.com/manyougz/velotype>.
