use std::sync::Arc;

use gpui::*;
use gpui_markdown_editor::{
    MarkdownEditor, MarkdownEditorEvent, MarkdownEditorMode, MarkdownEditorOptions,
    MarkdownEditorTheme, default_key_bindings,
};

const SAMPLE: &str = r#"# GPUI Markdown Editor

This editor is a **standalone GPUI component** extracted from Velotype.

- Rendered block editing
- Raw Markdown source editing
- Undo / redo and source-stable mode switching

> [!NOTE]
> Use the toolbar or `Ctrl/Cmd+Tab` to switch modes. Theme changes are instance-local.

| API | Purpose |
| --- | --- |
| `markdown()` | Read canonical Markdown |
| `replace_markdown()` | Replace the document |
| `set_mode()` | Select rendered or source mode |

```rust
let editor = cx.new(|cx| {
    MarkdownEditor::new(markdown, options, cx)
});
```
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DemoTheme {
    Dark,
    Light,
}

struct Demo {
    editor: Entity<MarkdownEditor>,
    revision: u64,
    theme: DemoTheme,
}

impl Demo {
    fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| MarkdownEditor::new(SAMPLE, MarkdownEditorOptions::default(), cx));
        cx.subscribe(&editor, Self::on_editor_event).detach();
        Self {
            editor,
            revision: 0,
            theme: DemoTheme::Dark,
        }
    }

    fn on_editor_event(
        &mut self,
        _editor: Entity<MarkdownEditor>,
        event: &MarkdownEditorEvent,
        cx: &mut Context<Self>,
    ) {
        if let MarkdownEditorEvent::Changed { revision } = event {
            self.revision = *revision;
        }
        cx.notify();
    }

    fn set_rendered(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.set_mode(MarkdownEditorMode::Rendered, cx)
        });
    }

    fn set_source(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.set_mode(MarkdownEditorMode::Source, cx)
        });
    }

    fn set_dark_theme(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.set_theme(Arc::new(MarkdownEditorTheme::default_theme()), cx)
        });
        self.theme = DemoTheme::Dark;
        cx.notify();
    }

    fn set_light_theme(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.set_theme(Arc::new(MarkdownEditorTheme::light_theme()), cx)
        });
        self.theme = DemoTheme::Light;
        cx.notify();
    }

    fn toolbar_button(
        &self,
        id: &'static str,
        label: &'static str,
        active: bool,
        on_click: fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>),
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(id)
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(6.0))
            .text_size(px(13.0))
            .text_color(if active {
                Hsla::from(rgba(0xffffffff))
            } else {
                Hsla::from(rgba(0xc8d1e0ff))
            })
            .bg(if active {
                rgba(0x4969d8ff)
            } else {
                rgba(0x2b3344ff)
            })
            .hover(|style| style.bg(rgba(0x3b465cff)))
            .cursor_pointer()
            .child(label)
            .on_click(cx.listener(on_click))
            .into_any_element()
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.editor.read(cx).mode();
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgba(0x111722ff))
            .child(
                div()
                    .h(px(52.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .border_b(px(1.0))
                    .border_color(rgba(0x30394aff))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(self.toolbar_button(
                                "rendered-mode",
                                "Rendered",
                                mode == MarkdownEditorMode::Rendered,
                                Self::set_rendered,
                                cx,
                            ))
                            .child(self.toolbar_button(
                                "source-mode",
                                "Source",
                                mode == MarkdownEditorMode::Source,
                                Self::set_source,
                                cx,
                            ))
                            .child(
                                div()
                                    .w(px(1.0))
                                    .h(px(24.0))
                                    .mx(px(4.0))
                                    .bg(rgba(0x3a4558ff)),
                            )
                            .child(self.toolbar_button(
                                "dark-theme",
                                "Dark",
                                self.theme == DemoTheme::Dark,
                                Self::set_dark_theme,
                                cx,
                            ))
                            .child(self.toolbar_button(
                                "light-theme",
                                "Light",
                                self.theme == DemoTheme::Light,
                                Self::set_light_theme,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgba(0x8d9bb2ff))
                            .child(format!("revision {}", self.revision)),
                    ),
            )
            .child(div().flex_1().min_h(px(0.0)).child(self.editor.clone()))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys(default_key_bindings());
        let bounds = Bounds::centered(None, size(px(1080.0), px(720.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("GPUI Markdown Editor".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(Demo::new),
            )
            .expect("failed to open demo window");
        window
            .update(cx, |_demo, window, _cx| window.activate_window())
            .expect("failed to activate demo window");
        eprintln!("GPUI Markdown Editor demo ready");
    });
}
