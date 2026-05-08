use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::{collections::VecDeque, time::Duration};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct UiApp {
    // terminal: ratatui::DefaultTerminal,
    selected_peer: usize,
    logs: VecDeque<String>,
    log_rx: mpsc::Receiver<String>,
}
impl UiApp {
    pub fn new(log_rx: mpsc::Receiver<String>) -> Self {
        Self {
            selected_peer: 0,
            logs: VecDeque::new(),
            log_rx,
        }
    }
    pub async fn run(&mut self, app: &App) {
        let mut terminal = ratatui::init();

        loop {
            if !app.running.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            tokio::select! {
                Some(event) = self.log_rx.recv() => {
                    self.logs.push_back(event);
                    if self.logs.len() > 10 {
                        self.logs.pop_front();
                    }
                }
                tick = tokio::task::spawn(async {event::poll(Duration::from_millis(100))}) =>{
                    if let Ok(tick) = tick{
                        match tick {
                            Ok(true) => {
                                if let Ok(event) = event::read() {
                                    if self.handle_event(app, event).await {
                                        break;
                                    }
                                }
                            }
                            Ok(false) => {}
                            _ => {
                                unreachable!();
                            }
                        }
                        terminal.draw(|f| {
                            self.render(f, app);
                        }).unwrap();
                    }else{
                        unreachable!();
                    }
                }
            }
        }

        ratatui::restore();
    }

    pub fn render(&self, f: &mut ratatui::Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Fill(1)])
            .split(f.area());

        self.render_peer_list(f, app, chunks[0]);
        self.render_log(f, chunks[1]);
    }
    fn render_peer_list(&self, f: &mut Frame, app: &App, area: Rect) {
        let selected_idx = self.selected_peer;

        let peers = app.peers.try_read();
        let peer_items: Vec<ListItem> = if let Ok(peers) = peers {
            peers
                .iter()
                .enumerate()
                .map(|(i, peer)| {
                    let is_selected = i == selected_idx;
                    let volume_bar = Self::render_volume_bar(
                        peer.volume.load(std::sync::atomic::Ordering::Relaxed),
                    );
                    let text = Text::from(vec![
                        Line::from(Span::raw(format!(" {}: {}", peer.name, peer.addr.ip()))),
                        Line::from(vec![
                            if is_selected {
                                Span::styled("> ", Color::Yellow)
                            } else {
                                Span::raw("  ")
                            },
                            Span::raw("  Volume: "),
                            Span::styled(volume_bar, Color::Yellow),
                            Span::raw(format!(
                                " {:04}%",
                                peer.volume.load(std::sync::atomic::Ordering::Relaxed) / 10
                            )),
                        ]),
                    ]);

                    ListItem::new(text)
                })
                .collect()
        } else {
            vec![]
        };

        let peer_list =
            List::new(peer_items).block(Block::default().title("Peers").borders(Borders::ALL));

        f.render_widget(peer_list, area);
    }

    fn render_log(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Fill(1)])
            .split(area);

        let local_info = Text::from(vec![
            Line::from("Controls:"),
            Line::from("  Ctrl+C - Quit"),
            Line::from("  Ctrl+R - Refresh peers"),
            Line::from("  ↑/↓ - Select peer"),
            Line::from("  ←/→ - Adjust volume"),
        ]);

        let local_widget = Paragraph::new(local_info)
            .block(Block::default().title("Status").borders(Borders::ALL));

        let app_logs = self.logs.clone();
        let log_text = if app_logs.is_empty() {
            Text::from("No messages yet...")
        } else {
            let lines: Vec<Line> = app_logs
                .iter()
                .take(15)
                .map(|msg| Line::from(msg.as_str()))
                .collect();
            Text::from(lines)
        };

        let log_widget = Paragraph::new(log_text)
            .block(Block::default().title("Log").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(local_widget, chunks[0]);
        f.render_widget(log_widget, chunks[1]);
    }

    fn render_volume_bar(volume: u16) -> String {
        let filled = (volume / 100) as usize;
        let empty = 20usize.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    async fn handle_event(&mut self, app: &App, event: Event) -> bool {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        return true;
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.peers.write().await.clear();
                    }
                }
                KeyCode::Up if key.kind == KeyEventKind::Press => {
                    self.selected_peer = self.selected_peer.saturating_sub(1);
                }
                KeyCode::Down if key.kind == KeyEventKind::Press => {
                    let peers = app.peers.read().await;
                    self.selected_peer = (self.selected_peer + 1).min(peers.len())
                }
                KeyCode::Left => {
                    let peers = app.peers.read().await;
                    self.selected_peer = self.selected_peer.min(peers.len().saturating_sub(1));

                    if let Some(peer) = peers.get(self.selected_peer) {
                        let volume = peer.volume.load(std::sync::atomic::Ordering::Relaxed);
                        peer.volume.store(
                            volume.saturating_sub(10),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                }
                KeyCode::Right => {
                    let peers = app.peers.read().await;
                    self.selected_peer = self.selected_peer.min(peers.len().saturating_sub(1));

                    if let Some(peer) = peers.get(self.selected_peer) {
                        let volume = peer.volume.load(std::sync::atomic::Ordering::Relaxed);
                        peer.volume.store(
                            volume.saturating_add(10),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        };
        false
    }
}
