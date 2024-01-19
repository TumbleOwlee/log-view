use cursive::event::Key;
use cursive::theme::Style;
use cursive::traits::{Nameable, Resizable};
use cursive::utils::span::SpannedString;
use cursive::view::{ScrollStrategy, Scrollable};
use cursive::{
    views::{
        DummyView, EditView, EnableableView, HideableView, LinearLayout, NamedView, OnEventView,
        Panel, ResizedView, ScrollView, SelectView, TextContent, TextView,
    },
    CbSink,
};
use cursive::{Cursive, CursiveExt};

use std::io::prelude::Write;

use crate::buffer::SourceBuffer;
use crate::error::Error;
use crate::source::Source;
use crate::string::ColoredString;

type HistoryHide = HideableView<LinearLayout>;
type HistoryScroll = ScrollView<ResizedView<NamedView<SelectView>>>;
type ContentScroll = ScrollView<ResizedView<NamedView<SelectView>>>;
type ContentEvent = OnEventView<NamedView<ContentScroll>>;
type ContentEnableable = EnableableView<NamedView<ContentEvent>>;
type CommandHide = HideableView<LinearLayout>;

const GLOBAL_ONEVENT: &str = "global-onevent";
const CONTENT_VIEW: &str = "content-view";
const CONTENT_SCROLL: &str = "content-scroll";
const CONTENT_ENABLE: &str = "content-enable";
const CONTENT_EVENT: &str = "content-event";
const HISTORY_VIEW: &str = "history-view";
const HISTORY_SCROLL: &str = "history-scroll";
const HISTORY_HIDE: &str = "history-hide";
const COMMAND_VIEW: &str = "command-view";
const COMMAND_ONEVENT: &str = "command-onevent";
const COMMAND_HIDE: &str = "command-hide";
const ERROR_VIEW: &str = "error-view";
const ERROR_HIDE: &str = "error-hide";
const HISTORY_LEN: usize = 50;

enum Event {
    Clear,
    Update(String),
}

pub enum Mode {
    RetainColors,
    RemoveColors,
    SkipColorCheck,
}

pub struct Tui {
    siv: Cursive,
    cb_sink: CbSink,
    error: TextContent,
    color_mode: Mode,
    history: Option<String>,
}

impl Tui {
    pub fn new() -> Self {
        let siv = Cursive::default();
        let cb_sink = siv.cb_sink().clone();
        Self {
            siv,
            cb_sink,
            error: TextContent::new(""),
            color_mode: Mode::SkipColorCheck,
            history: None,
        }
    }

    pub fn set_color_mode(mut self, mode: Mode) -> Self {
        self.color_mode = mode;
        self
    }

    pub fn set_history_path(&mut self, path: String) {
        self.history = Some(path);
    }

    fn init_events(&mut self) {
        if let Some(mut v) = self.siv.find_name::<ContentEvent>(CONTENT_EVENT) {
            v.set_on_pre_event('G', |siv| {
                if let Some(mut v) = siv.find_name::<SelectView<String>>(CONTENT_VIEW) {
                    let len = v.len();
                    v.set_selection(len - 1)(siv);
                }
                if let Some(mut v) = siv.find_name::<ContentScroll>(CONTENT_SCROLL) {
                    v.scroll_to_bottom();
                    v.set_scroll_strategy(ScrollStrategy::StickToBottom);
                }
            });
            v.set_on_pre_event('g', |siv| {
                if let Some(mut v) = siv.find_name::<SelectView<String>>(CONTENT_VIEW) {
                    v.set_selection(0)(siv);
                }
                if let Some(mut v) = siv.find_name::<ContentScroll>(CONTENT_SCROLL) {
                    v.scroll_to_top();
                    v.set_scroll_strategy(ScrollStrategy::StickToTop);
                }
            });
            v.set_on_pre_event('0', |siv| {
                if let Some(mut v) = siv.find_name::<ContentScroll>(CONTENT_SCROLL) {
                    v.scroll_to_left();
                }
            });
            v.set_on_pre_event('$', |siv| {
                if let Some(mut v) = siv.find_name::<ContentScroll>(CONTENT_SCROLL) {
                    v.scroll_to_right();
                }
            });
        }
    }

    fn build_ui(&mut self, tx: std::sync::mpsc::Sender<Event>) {
        let path = self.history.take();
        self.siv.add_fullscreen_layer(
            OnEventView::new(ResizedView::with_full_screen(
                LinearLayout::vertical()
                    .child(ResizedView::with_full_height(
                        Panel::new(
                            EnableableView::new(
                                OnEventView::new(
                                    SelectView::<String>::new()
                                        .with_name(CONTENT_VIEW)
                                        .full_width()
                                        .scrollable()
                                        .show_scrollbars(false)
                                        .scroll_y(true)
                                        .scroll_x(true)
                                        .scroll_strategy(ScrollStrategy::StickToBottom)
                                        .with_name(CONTENT_SCROLL),
                                )
                                .with_name(CONTENT_EVENT),
                            )
                            .with_name(CONTENT_ENABLE),
                        )
                        .title("Log View"),
                    ))
                    .child(
                        HideableView::new(
                            LinearLayout::horizontal()
                                .child(DummyView)
                                .child(
                                    SelectView::<String>::new()
                                        .on_submit(Tui::on_submit_history)
                                        .with_name(HISTORY_VIEW)
                                        .full_width()
                                        .scrollable()
                                        .with_name(HISTORY_SCROLL)
                                        .full_width()
                                        .max_height(5),
                                )
                                .child(DummyView),
                        )
                        .with_name(HISTORY_HIDE),
                    )
                    .child(
                        HideableView::new(
                            LinearLayout::horizontal()
                                .child(DummyView)
                                .child(
                                    OnEventView::new(
                                        EditView::new()
                                            .on_submit(move |s, cmd| {
                                                Tui::on_submit_command(s, cmd, &tx)
                                            })
                                            .with_name(COMMAND_VIEW)
                                            .fixed_height(1)
                                            .full_width(),
                                    )
                                    .on_pre_event(Key::Tab, Tui::on_show_history)
                                    .with_name(COMMAND_ONEVENT),
                                )
                                .child(DummyView),
                        )
                        .with_name(COMMAND_HIDE),
                    )
                    .child(
                        HideableView::new(
                            LinearLayout::horizontal()
                                .child(DummyView)
                                .child(
                                    TextView::new_with_content(self.error.clone())
                                        .with_name(ERROR_VIEW)
                                        .fixed_height(1),
                                )
                                .child(DummyView),
                        )
                        .with_name(ERROR_HIDE),
                    ),
            ))
            .on_pre_event(Key::Esc, move |siv| Tui::quit(siv, &path))
            .with_name(GLOBAL_ONEVENT),
        );

        if let Some(mut v) = self.siv.find_name::<HistoryHide>(HISTORY_HIDE) {
            v.hide();
        }

        self.init_events();
        self.siv.focus_name(COMMAND_VIEW).unwrap();
        self.siv.set_fps(30);
    }

    pub fn run(mut self, source: Source<String>) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.build_ui(tx);
        match self.color_mode {
            Mode::SkipColorCheck => self.spawn_update(source, rx, ColoredString::unstyled),
            Mode::RemoveColors => self.spawn_update(source, rx, ColoredString::plain),
            Mode::RetainColors => self.spawn_update(source, rx, ColoredString::styled),
        };

        self.siv.run();
    }

    fn select_view_append<T: 'static>(
        siv: &mut Cursive,
        id: &str,
        label: SpannedString<Style>,
        value: T,
    ) {
        if let Some(mut v) = siv.find_name::<SelectView<T>>(id) {
            let len = v.len();
            v.add_item(label, value);
            v.set_selection(len)(siv);
        } else {
            siv.quit();
        }
    }

    fn select_view_clear<T: 'static>(siv: &mut Cursive, id: &str) {
        if let Some(mut v) = siv.find_name::<SelectView<T>>(id) {
            v.clear();
        } else {
            siv.quit();
        }
    }

    fn spawn_update(
        &mut self,
        source: Source<String>,
        rx: std::sync::mpsc::Receiver<Event>,
        parser: impl Fn(&str) -> SpannedString<Style> + Send + Copy + 'static,
    ) {
        let cb_sink = self.cb_sink.clone();
        let error = self.error.clone();
        std::thread::spawn(move || {
            let mut lines = 0;
            let mut buffer: SourceBuffer<String> = SourceBuffer::new(source);
            let mut filter: Option<regex::Regex> = None;
            loop {
                if let Some(s) = buffer.update() {
                    if let Some(ref r) = filter {
                        if r.is_match(&s) {
                            let s: String = s;
                            if cb_sink
                                .send(Box::new(move |siv| {
                                    Tui::select_view_append::<String>(
                                        siv,
                                        CONTENT_VIEW,
                                        parser(&s),
                                        Default::default(),
                                    )
                                }))
                                .is_err()
                            {
                                return;
                            }
                            lines += 1;
                        }
                    } else if filter.is_none() {
                        if cb_sink
                            .send(Box::new(move |siv| {
                                Tui::select_view_append::<String>(
                                    siv,
                                    CONTENT_VIEW,
                                    parser(&s),
                                    Default::default(),
                                )
                            }))
                            .is_err()
                        {
                            return;
                        }
                        lines += 1;
                    }
                }
                if lines > (2 * 1024) {
                    if cb_sink
                        .send(Box::new(|siv| {
                            Tui::select_view_clear::<String>(siv, CONTENT_VIEW)
                        }))
                        .is_err()
                    {
                        return;
                    }
                    lines = 0;
                }
                if let Ok(ev) = rx.try_recv() {
                    if cb_sink
                        .send(Box::new(|siv| {
                            Tui::select_view_clear::<String>(siv, CONTENT_VIEW)
                        }))
                        .is_err()
                    {
                        return;
                    }
                    lines = 0;
                    match ev {
                        Event::Clear => {
                            filter = None;
                        }
                        Event::Update(s) => match regex::Regex::new(&s) {
                            Ok(r) => filter = Some(r),
                            Err(e) => {
                                error.set_content(format!("{:?}", e));
                            }
                        },
                    }
                }
                if lines == 0 {
                    for item in buffer.iter() {
                        let item: String = item.into();
                        if let Some(ref r) = filter {
                            if r.is_match(&item) {
                                if cb_sink
                                    .send(Box::new(move |siv| {
                                        Tui::select_view_append::<String>(
                                            siv,
                                            CONTENT_VIEW,
                                            parser(&item),
                                            Default::default(),
                                        )
                                    }))
                                    .is_err()
                                {
                                    return;
                                }
                                lines += 1;
                            }
                        } else if filter.is_none() {
                            if cb_sink
                                .send(Box::new(move |siv| {
                                    Tui::select_view_append::<String>(
                                        siv,
                                        CONTENT_VIEW,
                                        parser(&item),
                                        Default::default(),
                                    )
                                }))
                                .is_err()
                            {
                                return;
                            }
                            lines += 1;
                        }
                    }
                } else {
                    std::thread::sleep(std::time::Duration::new(0, 200000));
                }
            }
        });
    }

    pub fn use_default_theme(&mut self) {
        self.siv
            .load_toml(include_str!("../theme/style.toml"))
            .unwrap();
    }

    pub fn use_custom_theme(&mut self, file: &str) -> Result<(), Error> {
        self.siv
            .load_theme_file(file)
            .map_err(|_| Error::CustomThemeFailed(file.to_owned()))
    }

    fn on_show_history(siv: &mut Cursive) {
        if !siv
            .find_name::<SelectView>(HISTORY_VIEW)
            .unwrap()
            .is_empty()
        {
            if let Some(mut v) = siv.find_name::<HistoryHide>(HISTORY_HIDE) {
                v.unhide();
            }
            if let Some(mut v) = siv.find_name::<HistoryScroll>(HISTORY_SCROLL) {
                v.scroll_to_bottom();
            }
            if let Some(mut v) = siv.find_name::<SelectView>(HISTORY_VIEW) {
                let id = v.len() - 1;
                v.set_selection(id);
            }
            siv.focus_name(HISTORY_VIEW).unwrap();
            if let Some(mut v) = siv.find_name::<CommandHide>(COMMAND_HIDE) {
                v.hide();
            }
            if let Some(mut v) = siv.find_name::<ContentEnableable>(CONTENT_ENABLE) {
                v.disable();
            }
        }
    }

    fn on_submit_command(siv: &mut Cursive, cmd: &str, tx: &std::sync::mpsc::Sender<Event>) {
        if let Some(mut v) = siv.find_name::<TextView>(ERROR_VIEW) {
            v.set_content("");
        }
        if cmd.is_empty() {
            tx.send(Event::Clear).unwrap();
        } else {
            if let Some(mut v) = siv.find_name::<SelectView>(HISTORY_VIEW) {
                let mut id = 0;
                for (_, value) in v.iter() {
                    if value == cmd {
                        break;
                    }
                    id += 1;
                }
                if id != v.len() {
                    v.remove_item(id);
                } else if v.len() == HISTORY_LEN {
                    v.remove_item(0);
                }

                v.add_item_str(cmd);
                tx.send(Event::Update(cmd.to_owned())).unwrap();
            }
            if let Some(mut v) = siv.find_name::<EditView>(COMMAND_VIEW) {
                v.set_content("");
            }
            if let Some(mut v) = siv.find_name::<ContentScroll>(CONTENT_SCROLL) {
                v.set_scroll_strategy(ScrollStrategy::StickToBottom);
            }
        }
    }

    fn on_submit_history(siv: &mut Cursive, _cmd: &str) {
        if let Some(mut v) = siv.find_name::<CommandHide>(COMMAND_HIDE) {
            v.unhide();
        }
        if let Some(mut v) = siv.find_name::<SelectView>(HISTORY_VIEW) {
            if let Some(id) = v.selected_id() {
                let item = v.get_item(id).unwrap().1.clone();
                v.remove_item(id);
                v.add_item_str(item.clone());
                if let Some(mut c) = siv.find_name::<EditView>(COMMAND_VIEW) {
                    c.set_content(item);
                }
                if let Some(mut h) = siv.find_name::<HistoryHide>(HISTORY_HIDE) {
                    h.hide();
                }
                siv.focus_name(COMMAND_VIEW).unwrap();
            }
        }
        if let Some(mut v) = siv.find_name::<ContentEnableable>(CONTENT_ENABLE) {
            v.enable();
        }
    }

    fn quit(siv: &mut Cursive, path: &Option<String>) {
        if let Some(p) = path {
            if let Some(v) = siv.find_name::<SelectView>(HISTORY_VIEW) {
                if let Ok(mut f) = std::fs::File::create(p) {
                    if f.write_fmt(format_args!("[History]\n")).is_ok() {
                        for item in v.iter() {
                            if f.write_fmt(format_args!("{}\n", item.0)).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
        siv.quit();
    }
}
