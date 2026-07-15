use std::sync::Arc;

use gpui::{AppContext, TestAppContext};

use super::Editor;
use crate::{
    MarkdownEditorEnvironment, MarkdownEditorMode, MarkdownEditorOptions, MarkdownEditorTheme,
};

#[gpui::test]
async fn rendered_source_round_trip_preserves_markdown(cx: &mut TestAppContext) {
    let markdown = "# Title\n\nParagraph with **bold**.\n\n| A | B |\n| --- | --- |\n| 1 | 2 |";
    let editor = cx.new(|cx| Editor::new(markdown, MarkdownEditorOptions::default(), cx));

    editor.update(cx, |editor, cx| {
        assert_eq!(editor.markdown(cx), markdown);
        editor.set_mode(MarkdownEditorMode::Source, cx);
        assert_eq!(editor.markdown(cx), markdown);
        editor.set_mode(MarkdownEditorMode::Rendered, cx);
        assert_eq!(editor.markdown(cx), markdown);
    });
}

#[gpui::test]
async fn replace_markdown_preserves_the_selected_mode(cx: &mut TestAppContext) {
    let options = MarkdownEditorOptions {
        mode: MarkdownEditorMode::Source,
        ..MarkdownEditorOptions::default()
    };
    let editor = cx.new(|cx| Editor::new("alpha", options, cx));

    editor.update(cx, |editor, cx| {
        editor.replace_markdown("beta\ngamma", cx);
        assert_eq!(editor.mode(), MarkdownEditorMode::Source);
        assert_eq!(editor.markdown(cx), "beta\ngamma");
    });
}

#[gpui::test]
async fn theme_and_strings_are_isolated_per_instance(cx: &mut TestAppContext) {
    let mut first_theme = MarkdownEditorTheme::default_theme();
    first_theme.name = "first".into();
    let mut second_theme = MarkdownEditorTheme::default_theme();
    second_theme.name = "second".into();

    let first = cx.new(|cx| {
        Editor::new(
            "one",
            MarkdownEditorOptions {
                environment: MarkdownEditorEnvironment {
                    theme: Arc::new(first_theme),
                    ..MarkdownEditorEnvironment::default()
                },
                ..MarkdownEditorOptions::default()
            },
            cx,
        )
    });
    let second = cx.new(|cx| {
        Editor::new(
            "two",
            MarkdownEditorOptions {
                environment: MarkdownEditorEnvironment {
                    theme: Arc::new(second_theme),
                    ..MarkdownEditorEnvironment::default()
                },
                ..MarkdownEditorOptions::default()
            },
            cx,
        )
    });

    assert_eq!(
        first.read_with(cx, |editor, _| editor.environment().theme.name.clone()),
        "first"
    );
    assert_eq!(
        second.read_with(cx, |editor, _| editor.environment().theme.name.clone()),
        "second"
    );
}
