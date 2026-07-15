# GPUI Markdown Editor

[English](README.md) | **简体中文**

> [!IMPORTANT]
> ## 项目由来与署名
> 本项目由 [`manyougz` 创建的 Velotype](https://github.com/manyougz/velotype) 的 Markdown 编辑核心抽离、改造而来。Velotype 是一个使用 Rust + GPUI 实现的原生 Markdown 编辑器；本组件的编辑器架构与实现建立在该项目工作的基础上，原项目及其作者应获得相应署名。
>
> 本仓库将这部分能力整理为可独立复用的 GPUI 组件，并不替代 Velotype 完整的应用外壳。项目保留了原始 Apache License 2.0 和版权声明，详见 [LICENSE-APACHE](LICENSE-APACHE)。

## 项目介绍

`gpui-markdown-editor` 是一个使用 Rust 和 [GPUI](https://gpui.rs/) 实现的原生 Markdown 编辑组件。它为同一份 Markdown 文档提供两种编辑表示：

- **渲染模式（Rendered）**：基于块的所见即所得式编辑，同时保留 Markdown 语义。
- **源码模式（Source）**：在单一源码缓冲区中直接编辑原始 Markdown，可选择显示行号。

切换模式时，组件通过 canonical Markdown 完成序列化和目标表示重建，并根据源码偏移恢复选区。对于当前无法安全结构化编辑的语法，组件会回退为 Raw Markdown 块，使内容仍然可见、可编辑，并避免在往返转换时静默丢失。

## 主要功能

- 渲染和源码两种编辑模式。
- 段落、标题、列表、任务列表、引用、Callout 和分隔线。
- 强调、链接、引用链接、行内代码、行内 HTML 和数学公式投影。
- 原生表格编辑和表格行列操作。
- 围栏代码块和可选的 Tree-sitter 语法高亮。
- 图片、HTML 块、LaTeX、脚注和 Mermaid 图表。
- 面向 IME、CJK 文本和 Emoji 的 UTF-8/UTF-16 选区映射。
- 包含跨块编辑的文档级撤销和重做。
- 在模式切换期间保持稳定的源码偏移选区 API。
- 每个编辑器实例独立配置主题、文案、显示选项、文档目录和图片粘贴策略。
- 向宿主发送文档变化、模式变化、选区、链接请求和错误事件。

## 组件边界

该 crate 负责 Markdown 编辑状态和渲染；应用级行为由宿主程序负责：

| 组件负责 | 宿主负责 |
| --- | --- |
| Markdown 文档模型 | 打开、保存和另存为流程 |
| 渲染/源码模式转换 | dirty 状态和关闭确认 |
| 编辑、选区、撤销/重做 | 窗口和工作区管理 |
| Markdown 和块渲染 | 菜单及应用偏好设置 |
| 发送链接激活请求 | 实际打开外部链接 |
| 图片插入事务 | 编码图片的落盘或上传策略 |
| 使用主题和文案 | 加载、选择主题和语言 |
| 发送错误事件 | 应用级错误对话框 |

因此，现有 GPUI 应用可以直接嵌入编辑器，而不必同时引入 Velotype 的窗口外观、工作区、持久化、导出或更新系统。

## 仓库结构

```text
.
├── Cargo.toml                         # Workspace 和 Demo 应用
├── src/main.rs                        # 可运行的 GPUI Demo
└── crates/gpui-markdown-editor/
    ├── Cargo.toml                     # 可复用组件 crate
    └── src/
        ├── lib.rs                     # 公共入口
        ├── api.rs                     # 模式、命令、事件和选区
        ├── environment.rs             # 实例级配置
        ├── host.rs                    # 图片粘贴宿主接口
        ├── editor/                    # 文档和编辑器运行时
        └── components/                # Block 与 Markdown 渲染运行时
```

## 环境要求

- 支持 Rust 2024 Edition 的当前 stable Rust 工具链。
- GPUI `0.2.2` 及其平台依赖。

不同平台的准备方式请参考 [GPUI 文档](https://docs.rs/gpui/0.2.2/gpui/)。

## 运行 Demo

在仓库根目录执行：

```bash
cargo run -p gpui-markdown-editor-demo
```

Demo 会在独立 GPUI 窗口中嵌入组件。可以点击 **Rendered** 和 **Source** 按钮，或使用 `Ctrl/Cmd+Tab` 切换模式。

## 在第三方项目中引用

可以直接通过 GitHub 引用组件：

```toml
[dependencies]
gpui = { version = "0.2.2", features = ["runtime_shaders"] }
gpui-markdown-editor = { git = "https://github.com/yuWorm/gpui-markdeditor.git", package = "gpui-markdown-editor" }
```

Cargo 会在远程 Workspace 中找到 `gpui-markdown-editor` package。应用项目应提交 `Cargo.lock`，从而固定已经解析出的 Git revision；库项目或需要明确固定版本的项目可以增加 `rev = "<commit-sha>"`。

如果使用 SSH 拉取，可以写成：

```toml
gpui-markdown-editor = { git = "ssh://git@github.com/yuWorm/gpui-markdeditor.git", package = "gpui-markdown-editor" }
```

本地开发时可以改用路径依赖：

```toml
gpui-markdown-editor = { path = "../gpui-markdeditor/crates/gpui-markdown-editor" }
```

请根据本仓库的实际检出位置调整路径。

## 基础接入

应用启动时注册一次组件默认快捷键：

```rust
use gpui::*;
use gpui_markdown_editor::default_key_bindings;

Application::new().run(|cx: &mut App| {
    cx.bind_keys(default_key_bindings());
    // 在此创建应用窗口。
});
```

在宿主 View 中创建编辑器 Entity 并订阅事件：

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
                // 由宿主校验并打开 request.open_target。
            }
            MarkdownEditorEvent::Error { message } => {
                // 使用宿主应用的 UI 显示错误。
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

完整的窗口和工具栏示例见 [`src/main.rs`](src/main.rs)。

## 读取和替换 Markdown

```rust
// 宿主需要保存时，读取当前 canonical Markdown。
let markdown = self.editor.read(cx).markdown(cx);

// 替换整份文档，并重置撤销/重做历史。
self.editor.update(cx, |editor, cx| {
    editor.replace_markdown(next_markdown, cx);
});
```

`Changed` 事件有意只携带 revision，而不是完整 Markdown。这可以避免每次按键都分配并序列化整份文档。宿主应只在保存、导出等确实需要文本时调用 `markdown()`。

## 控制编辑模式

```rust
use gpui_markdown_editor::{EditorCommand, MarkdownEditorMode};

self.editor.update(cx, |editor, cx| {
    editor.set_mode(MarkdownEditorMode::Source, cx);
    editor.execute(EditorCommand::Undo, cx);
    editor.execute(EditorCommand::Redo, cx);
    editor.toggle_mode(cx);
});
```

可用文档命令包括 `Undo`、`Redo`、`ToggleMode` 和 `SetMode`。

## 配置单个编辑器实例

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

运行时可使用 `set_environment` 替换配置。环境数据属于单个编辑器实例，组件不会读取 Velotype 应用全局状态。

## 图片粘贴策略

默认情况下，本地图片路径会直接写入 Markdown；剪贴板中的编码图片字节则要求宿主提供 `ImagePasteHandler`：

```rust
pub trait ImagePasteHandler: Send + Sync + 'static {
    fn materialize(
        &self,
        source: PastedImage,
        document_base_dir: Option<&Path>,
    ) -> anyhow::Result<ImageTarget>;
}
```

宿主可以通过这个边界把文件复制到文档旁、写入 `assets` 目录、上传到远程存储，或者拒绝该操作。该方法保持同步，使图片插入、选区更新和 undo capture 处在同一个事务中。

## Cargo Features

默认启用：

- `code-highlight-core`
- `code-highlight-official`
- `code-highlight-config`
- `html-native`

如需减少解析器依赖，可以关闭默认 Feature，只启用需要的能力：

```toml
gpui-markdown-editor = {
    path = "crates/gpui-markdown-editor",
    default-features = false,
    features = ["code-highlight-core", "html-native"]
}
```

## 测试

```bash
cargo check -p gpui-markdown-editor-demo
cargo test -p gpui-markdown-editor -- --test-threads=1
```

组件测试覆盖模式往返、Markdown 导入与序列化、块编辑、选区映射、表格、IME、撤销/重做和实例级配置。

## 开源协议

本项目采用 Apache License 2.0，详见 [LICENSE-APACHE](LICENSE-APACHE)。

项目保留 Velotype 原始版权声明与协议。原项目地址：<https://github.com/manyougz/velotype>。
