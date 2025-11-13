use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::app::App;
use crate::app::app::MachineKind;

pub fn render_app(area: Rect, buf: &mut Buffer, app: &App) {
    AppView { app }.render(area, buf);
}

struct AppView<'a> {
    app: &'a App,
}

impl<'a> Widget for AppView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split whole screen: top = machine, bottom = controls
        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .areas(area);

        // Further split top horizontally: [ expr | stack ]
        let [expr_area, stack_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(top);

        // ---- Title ----
        let machine_title = match self.app.machine {
            MachineKind::CK(_) => " CK Machine ",
            // MachineKind::CEK(_) => " CEK Machine ",
        };
        let title = Line::from(machine_title).bold().centered();

        // ---- Pull current state from the machine ----
        let (expr_text, stack_lines) = match &self.app.machine {
            MachineKind::CK(machine) => {
                let state = machine.state();
                let expr_text = state.current_expression.to_string();
                let stack_lines = state.stack_lines();
                (expr_text, stack_lines)
            } // later: MachineKind::CEK(m) => { ... }
        };

        // Left: current expression
        let expr_block = Block::bordered()
            .title(title.clone())
            .border_set(border::THICK);
        let expr_body = Text::from(vec![
            Line::from(format!("Step: {}", self.app.step)),
            Line::from(expr_text),
        ]);
        Paragraph::new(expr_body)
            .block(expr_block)
            .render(expr_area, buf);

        let stack_title = Line::from(" Stack ").bold().centered();

        let stack_block = Block::bordered()
            .title(stack_title)
            .border_set(border::THICK);

        let stack_body = Text::from(stack_lines.into_iter().map(Line::from).collect::<Vec<_>>());
        Paragraph::new(stack_body)
            .block(stack_block)
            .render(stack_area, buf);

        // ---- Bottom: controls ----
        let controls = Line::from(vec![
            " Step back ".into(),
            "<Left>".blue().bold(),
            " Step forward ".into(),
            "<Right>".blue().bold(),
            " Switch machine ".into(),
            "<M>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);

        let bottom_block = Block::bordered()
            .title_bottom(controls.centered())
            .border_set(border::THICK);

        bottom_block.render(bottom, buf);
    }
}
