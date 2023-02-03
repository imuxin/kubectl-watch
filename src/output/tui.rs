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
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

struct App {
    state: TableState,
    items: Vec<DynamicObject>,
    map: HashMap<String, Vec<DynamicObject>>,
    text: String,
}

impl App {
    fn new() -> App {
        App {
            state: TableState::default(),
            items: vec![],
            map: HashMap::new(),
            text: "no diff for first event".to_string(),
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

    fn get_header<'a>(&mut self) -> Vec<&'a str> {
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
        self.state.select(Some(i));
    }
}

pub async fn main_tui(chan: mpsc::Receiver<event::Msg>) -> anyhow::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app, chan).await;

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

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut chan: mpsc::Receiver<event::Msg>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Some(_msg) = chan.recv().await {
            match _msg {
                event::Msg::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    _ => {}
                },
                event::Msg::Obj(obj) => app.update(obj),
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(10), Constraint::Min(10)].as_ref())
        .split(f.size());

    draw_resources_event(f, app, chunks[0]);
    draw_diff(f, app, chunks[1]);
}

fn draw_resources_event<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let selected_style = Style::default().bg(Color::Black).fg(Color::LightRed);
    let header = Row::new(app.get_header())
        .style(Style::default().fg(Color::LightBlue))
        .height(1)
        .bottom_margin(0);

    let r = app.get_raws();
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
    f.render_stateful_widget(t, area, &mut app.state);
}

fn draw_diff<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let text = vec![Spans::from(Span::styled(
        app.text.as_str(),
        Style::default().fg(Color::Red),
    ))];
    let paragraph = Paragraph::new(text.clone())
        // .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Diff Content"))
        // .block(create_block("Diff Content"))
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
