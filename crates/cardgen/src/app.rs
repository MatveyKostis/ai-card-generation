use gpui::{
    App, Application, Bounds, Context, FocusHandle, Focusable, KeyBinding, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};

actions!(cardgen, [Generate, Quit]);

/// Prompt input backed by a simple single-line text buffer. Real input handling
/// (caret, selection, paste) lands with the UI polish slice; this focuses,
/// captures keystrokes, and Enter triggers Generate.
struct PromptInput {
    focus_handle: FocusHandle,
    content: String,
}

impl PromptInput {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
        }
    }

    fn on_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match &event.keystroke.key[..] {
            "enter" => cx.propagate(),
            "backspace" => {
                self.content.pop();
                cx.notify();
            }
            "space" => {
                self.content.push(' ');
                cx.notify();
            }
            "tab" => {}
            _ => {
                if let Some(text) = &event.keystroke.key_char {
                    if !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
                        self.content.push_str(text);
                        cx.notify();
                    }
                }
            }
        }
    }
}

impl Focusable for PromptInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PromptInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let placeholder = self.content.is_empty();
        div()
            .id("prompt-input")
            .track_focus(&self.focus_handle(cx))
            .on_key_down(cx.listener(Self::on_key_down))
            .bg(rgb(0x16181d))
            .border_1()
            .border_color(rgb(0x2e3340))
            .px_3()
            .py_2()
            .text_size(px(14.))
            .text_color(if placeholder {
                rgb(0x5c6370)
            } else {
                rgb(0xe6e6e6)
            })
            .child(if placeholder {
                "Describe a character… (Enter to generate)".to_string()
            } else {
                self.content.clone()
            })
    }
}

/// Root view: prompt input, generate button, and the returned card rendered as
/// structured output.
struct CardgenView {
    prompt: gpui::Entity<PromptInput>,
    prompt_focus: gpui::FocusHandle,
    card_json: Option<String>,
    error: Option<String>,
    busy: bool,
}

impl CardgenView {
    fn track_focus_handle(&self) -> gpui::FocusHandle {
        self.prompt_focus.clone()
    }

    fn on_generate(&mut self, _: &Generate, _: &mut Window, cx: &mut Context<Self>) {
        let prompt = self.prompt.read(cx).content.clone();
        if prompt.trim().is_empty() || self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        cx.notify();
        match crate::generate::generate_and_print(&prompt) {
            Ok(json) => self.card_json = Some(json),
            Err(error) => self.error = Some(format!("{error:#}")),
        }
        self.busy = false;
        cx.notify();
    }
}

impl Render for CardgenView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = if self.busy {
            "Generating…".to_string()
        } else if let Some(error) = &self.error {
            format!("error: {error}")
        } else {
            "prompt → model → card".to_string()
        };

        div()
            .key_context("CardgenView")
            .track_focus(&self.track_focus_handle())
            .on_action(cx.listener(Self::on_generate))
            .bg(rgb(0x0d0f13))
            .text_color(rgb(0xe6e6e6))
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(
                div()
                    .text_size(px(18.))
                    .child("AI Character Card Generator"),
            )
            .child(self.prompt.clone())
            .child(
                div()
                    .id("generate-button")
                    .bg(rgb(0x2f6feb))
                    .px_3()
                    .py_2()
                    .text_size(px(14.))
                    .child("Generate")
                    .hover(|style| style.bg(rgb(0x4c86f0)).cursor_pointer())
                    .on_click(cx.listener(|_, _, window, cx| {
                        window.dispatch_action(Box::new(Generate), cx);
                    })),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(0x8b949e))
                    .child(status),
            )
            .children(self.card_json.as_ref().map(|json| {
                div()
                    .id("card-output")
                    .flex_1()
                    .overflow_y_scroll()
                    .bg(rgb(0x16181d))
                    .border_1()
                    .border_color(rgb(0x2e3340))
                    .p_3()
                    .text_size(px(12.))
                    .child(json.clone())
            }))
    }
}

/// Opens the gpui window and runs the app loop.
pub fn run() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("enter", Generate, Some("CardgenView"))]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(640.), px(720.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    let prompt = cx.new(PromptInput::new);
                    let prompt_focus = prompt.read(cx).focus_handle.clone();
                    cx.new(|_cx| CardgenView {
                        prompt,
                        prompt_focus,
                        card_json: None,
                        error: None,
                        busy: false,
                    })
                },
            )
            .expect("failed to open window");

        window
            .update(cx, |view, window, _cx| {
                window.focus(&view.prompt_focus);
            })
            .expect("failed to focus prompt");
        cx.activate(true);
    });
}
