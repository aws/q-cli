use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};
pub fn landing_screen() -> Result<(), io::Error> {
    // enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // terminal.draw(|f| {
    //     let size = f.size();
    //     //.inner(&tui::layout::Margin { horizontal:0, vertical: f.size().height / 3 });
    //     let chunks = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .margin(1)
    //     .constraints(
    //         [
    //             Constraint::Percentage(6),
    //             Constraint::Percentage(41),
    //             Constraint::Percentage(6),
    //             Constraint::Percentage(41),
    //             Constraint::Percentage(6),

    //         ].as_ref()
    //     )
    //     .split(size);
    // let block = Block::default()
    //      .title("Block")
    //      .borders(Borders::ALL);
    // f.render_widget(block, chunks[1]);
    // let block = Block::default()
    //      .title("Block 2")
    //      .borders(Borders::ALL);
    // f.render_widget(block, chunks[3]);
    // })?;

    thread::sleep(Duration::from_millis(5000));

    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    // restore terminal
    // disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
    //     println!(
    //     "
    //                                              .--~~~~~~~~~~~~~------.
    //     ███████╗██╗ ██████╗                     /--===============------\\
    //     ██╔════╝██║██╔════╝                     | |⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺|     |
    //     █████╗  ██║██║  ███╗                    | | ~ ▮           |     |
    //     ██╔══╝  ██║██║   ██║                    | |               |     |
    //     ██║     ██║╚██████╔╝                    | |               |     |
    //     ╚═╝     ╚═╝ ╚═════╝                     | |_______________|     |
    //                                             |                   ::::|
    //   • Modern CLI autocomplete                 '======================='
    //   • Keyboard-driven UI for your dotfiles    //-'-'-'-'-'-'-'-'-'-'-\\\\
    //   • Discover themes & shell plugins        //_'_'_'_'_'_'_'_'_'_'_'_\\\\
    //   • Sync across all your devices           [-------------------------]
    //                                            \\_________________________/
    //     ");
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(ui)?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    let size = f.size();

    // Surrounding block
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Main block with round corners")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Top two inner blocks
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    // Top left inner block with green background
    let block = Block::default()
        .title(vec![
            Span::styled("With", Style::default().fg(Color::Yellow)),
            Span::from(" background"),
        ])
        .style(Style::default().bg(Color::Green));
    f.render_widget(block, top_chunks[0]);

    // Top right inner block with styled title aligned to the right
    let block = Block::default()
        .title(Span::styled(
            "Styled title",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    f.render_widget(block, top_chunks[1]);

    // Bottom two inner blocks
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Bottom left block with all default borders
    let block = Block::default().title("With borders").borders(Borders::ALL);
    f.render_widget(block, bottom_chunks[0]);

    // Bottom right block with styled left and right border
    let block = Block::default()
        .title("With styled borders and doubled borders")
        .border_style(Style::default().fg(Color::Cyan))
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(BorderType::Double);
    f.render_widget(block, bottom_chunks[1]);
}

pub fn prompt_to_run_fig() {
    println!("Run `fig` to get started...")
}
