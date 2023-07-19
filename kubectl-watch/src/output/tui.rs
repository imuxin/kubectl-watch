use crate::diff;
use crate::options;
use crate::output::{
    db::{Database, Memory, UID},
    event, utils,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kube::{api::DynamicObject, ResourceExt};
use std::{collections::HashMap, io};
use tokio::sync::mpsc;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    // text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
    Terminal,
};

impl UID for DynamicObject {
    fn resource_version(&self) -> String {
        ResourceExt::resource_version(self).unwrap()
    }
    fn uid(&self) -> String {
        let name = self.name_any();
        let namespace = self.namespace().unwrap_or_default();
        name + &namespace
    }
}

struct Controller<'a> {
    diff_tool: Box<dyn diff::Diff<'a>>,
    state: TableState,
    items: Vec<DynamicObject>,
    total_items: Vec<DynamicObject>,
    active_uid: Option<String>,
    database: Memory<DynamicObject>,
    l_diff: Paragraph<'a>,
    r_diff: Paragraph<'a>,
    scroll: u16,
    scroll_step: u16,
}

impl<'a> Controller<'a> {
    fn new(diff_tool: Box<dyn diff::Diff<'a>>) -> Controller<'a> {
        Controller {
            diff_tool: diff_tool,
            state: TableState::default(),
            items: vec![],
            total_items: vec![],
            active_uid: None,
            database: HashMap::new(),
            l_diff: Paragraph::new(""),
            r_diff: Paragraph::new(""),
            scroll: 0,
            scroll_step: 5,
        }
    }

    fn get_raws(&mut self) -> Vec<Vec<String>> {
        let mut raws = vec![];
        for (pos, item) in self.items.iter().enumerate() {
            raws.push(vec![
                (pos + 1).to_string(),
                item.namespace().unwrap_or("".to_owned()),
                item.name_any().to_owned(),
                utils::format_creation_since(item.creation_timestamp()),
                ResourceExt::resource_version(item).unwrap_or("".to_owned()),
            ])
        }
        return raws;
    }

    fn get_header<'b>(&mut self) -> Vec<&'b str> {
        return vec!["ID", "NAMESPACE", "NAME", "AGE", "REV"];
    }

    fn _reset_scroll(&mut self) {
        self.scroll = 0
    }

    fn _do_insert(&mut self, obj: DynamicObject) {
        self.database.do_insert(obj.clone());
        self.total_items.push(obj);
        self._refresh_items();
    }

    fn _refresh_items(&mut self) {
        match &self.active_uid {
            Some(uid) => {
                self.items = vec![];
                self.items
                    .extend_from_slice(self.database.items_of_uid(uid.clone()).unwrap());
            }
            None => {
                self.items = vec![];
                self.items.extend_from_slice(&self.total_items);
            }
        }
    }

    fn _do_diff(&mut self, select: usize) {
        if let Some(obj) = self.items.get(select) {
            (self.l_diff, self.r_diff) = self.diff_tool.tui_diff(self.database.sibling(obj), obj);
        }
    }

    pub fn next(&mut self) {
        self._reset_scroll();
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self._do_diff(i);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        self._reset_scroll();
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self._do_diff(i);
        self.state.select(Some(i));
    }

    pub fn enter(&mut self) {
        match self.state.selected() {
            Some(i) => {
                if let Some(obj) = self.items.clone().get(i) {
                    if self.active_uid == Some(UID::uid(obj)) {
                        return;
                    }
                    self.active_uid = Some(UID::uid(obj));
                    self._refresh_items();
                    let select = self.database.index_of(obj);
                    self._do_diff(select);
                    self.state.select(Some(select));
                }
            }
            None => {}
        };
    }

    pub fn escape(&mut self) {
        match &self.active_uid {
            None => {}
            Some(_) => {
                self.active_uid = None;

                match self.state.selected() {
                    Some(i) => {
                        if let Some(obj) = self.items.clone().get(i) {
                            self._refresh_items();
                            let mut select: usize = 0;
                            for (i, item) in self.items.iter().enumerate() {
                                if ResourceExt::resource_version(item)
                                    == ResourceExt::resource_version(obj)
                                {
                                    select = i;
                                }
                            }
                            self._do_diff(select);
                            self.state.select(Some(select));
                        }
                    }
                    None => {}
                };
            }
        }
    }

    pub fn page_home(&mut self) {
        self.scroll = 0;
    }

    pub fn page_up(&mut self) {
        if self.scroll < self.scroll_step {
            self.scroll = 0;
        } else {
            self.scroll -= self.scroll_step;
        }
    }

    pub fn page_down(&mut self) {
        self.scroll += self.scroll_step;
    }
}

pub async fn main_tui(app: &options::App, chan: mpsc::Receiver<event::Msg>) -> anyhow::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create ctrl and run it
    let diff_tool = diff::new(app);
    let ctrl = Controller::new(diff_tool);
    let res = run_tui(&mut terminal, ctrl, chan).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_tui<B: Backend>(
    terminal: &mut Terminal<B>,
    mut ctrl: Controller<'static>,
    mut chan: mpsc::Receiver<event::Msg>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut ctrl))?;

        if let Some(_msg) = chan.recv().await {
            match _msg {
                event::Msg::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => ctrl.next(),
                    KeyCode::Up | KeyCode::Char('k') => ctrl.previous(),
                    KeyCode::Enter => ctrl.enter(),
                    KeyCode::Esc => ctrl.escape(),
                    KeyCode::Home => ctrl.page_home(),
                    KeyCode::PageUp => ctrl.page_up(),
                    KeyCode::PageDown => ctrl.page_down(),
                    _ => {}
                },
                event::Msg::Obj(obj) => ctrl._do_insert(obj),
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, ctrl: &mut Controller) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(10), Constraint::Min(10)].as_ref())
        .split(f.size());

    draw_resources_event(f, ctrl, chunks[0]);
    draw_diff(f, ctrl, chunks[1]);
}

fn draw_resources_event<B>(f: &mut Frame<B>, ctrl: &mut Controller, area: Rect)
where
    B: Backend,
{
    let selected_style = Style::default().bg(Color::Black).fg(Color::LightRed);
    let header = Row::new(ctrl.get_header())
        .style(Style::default().fg(Color::LightBlue))
        .height(1)
        .bottom_margin(0);

    let r = ctrl.get_raws();
    let rows = r.iter().map(|item| {
        let height = &item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item
            .iter()
            .map(|c| Cell::from(c.to_string()).style(Style::default().fg(Color::White)));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Resources"))
        .highlight_style(selected_style)
        .widths(&[
            // Constraint::Percentage(10),
            Constraint::Percentage(5),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            // Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, area, &mut ctrl.state);
}

fn draw_diff<B>(f: &mut Frame<B>, ctrl: &mut Controller, area: Rect)
where
    B: Backend,
{
    f.render_widget(
        Block::default().borders(Borders::ALL).title("Diff Result"),
        area,
    );

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .direction(Direction::Horizontal)
        .split(area);

    f.render_widget(ctrl.l_diff.clone().scroll((ctrl.scroll, 0)), chunks[0]);
    f.render_widget(ctrl.r_diff.clone().scroll((ctrl.scroll, 0)), chunks[1]);
}
