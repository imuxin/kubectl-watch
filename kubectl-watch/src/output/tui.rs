use crate::diff;
use crate::options;
use crate::output;
use crate::output::event;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kube::{api::DynamicObject, ResourceExt};
use std::collections::HashMap;
use std::io;
use tokio::sync::mpsc;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame, Terminal,
};

struct Controller<'a> {
    diff_tool: Box<dyn diff::Diff<'a>>,
    state: TableState,
    items: Vec<DynamicObject>,
    map: HashMap<String, Vec<DynamicObject>>,
    l_diff: Paragraph<'a>,
    r_diff: Paragraph<'a>,
}

impl<'a> Controller<'a> {
    fn new(diff_tool: Box<dyn diff::Diff<'a>>) -> Controller<'a> {
        Controller {
            diff_tool: diff_tool,
            state: TableState::default(),
            items: vec![],
            map: HashMap::new(),
            l_diff: Paragraph::new(""),
            r_diff: Paragraph::new(""),
        }
    }

    fn get_raws(&mut self) -> Vec<Vec<String>> {
        let mut raws = vec![];
        for (pos, item) in self.items.iter().enumerate() {
            raws.push(vec![
                (pos + 1).to_string(),
                item.namespace().unwrap_or("".to_owned()),
                item.name_any().to_owned(),
                output::format_creation_since(item.creation_timestamp()),
                item.resource_version().unwrap_or("".to_owned()),
            ])
        }
        return raws;
    }

    fn get_header<'b>(&mut self) -> Vec<&'b str> {
        return vec!["ID", "NAMESPACE", "NAME", "AGE", "REV"];
    }

    fn update(&mut self, obj: DynamicObject) {
        let empty_list = Vec::<DynamicObject>::new();
        let name = obj.name_any();
        let namespace = obj.namespace().unwrap_or_default();
        let key = name + &namespace;
        self.map.entry(key.clone()).or_insert(empty_list);
        if let Some(list) = self.map.get_mut(&key.clone()) {
            list.push(obj.clone());
        }
        self.items.push(obj)
    }

    fn index(&mut self, select: usize) -> usize {
        let obj = self.items.get(select).unwrap();
        let name = obj.name_any();
        let namespace = obj.namespace().unwrap_or_default();
        let key = name + &namespace;
        for (i, item) in self.map.get(&key.clone()).unwrap().iter().enumerate() {
            if item.resource_version().unwrap() == obj.resource_version().unwrap() {
                return i;
            }
        }
        0
    }

    fn do_diff(&mut self, select: usize) {
        let pos = self.index(select);
        if pos == 0 {
            // set default diff msg
            self.l_diff = Paragraph::new(Span::from("no previous item to compare."))
                .wrap(Wrap { trim: true });
            self.r_diff = Paragraph::new("");
            return;
        }
        let obj = self.items.get(select).unwrap();
        let name = obj.name_any();
        let namespace = obj.namespace().unwrap_or_default();
        let key = name + &namespace;
        (self.l_diff, self.r_diff) = self.diff_tool.tui_diff(
            self.map.get(&key.clone()).unwrap().get(pos - 1).unwrap(),
            obj,
        )
    }

    pub fn next(&mut self) {
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
        self.do_diff(i);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
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
        self.do_diff(i);
        self.state.select(Some(i));
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
                    _ => {}
                },
                event::Msg::Obj(obj) => ctrl.update(obj),
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

    f.render_widget(ctrl.l_diff.clone(), chunks[0]);
    f.render_widget(ctrl.r_diff.clone(), chunks[1]);
}
