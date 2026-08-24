use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};

pub struct WheelData {
    pub label: &'static str,
    pub x: f64,
    pub y: f64,
}

impl WheelData {
    fn style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    }
}
pub struct WheelSection<'a> {
    wheels: &'a [WheelData],
    selected_index: usize,
}

pub fn default_wheels() -> Vec<WheelData> {
    vec![
        WheelData {
            label: "Lift (Shadows)",
            x: 0.0,
            y: 0.0,
        },
        WheelData {
            label: "Gamma (Midtones)",
            x: 0.1,
            y: -0.1,
        },
        WheelData {
            label: "Gain (Highlights)",
            x: 0.3,
            y: 0.5,
        },
    ]
}

impl<'a> WheelSection<'a> {
    pub fn new(wheels: &'a [WheelData], selected_index: usize) -> Self {
        Self {
            wheels,
            selected_index,
        }
    }
}

//TODO: Add slider to wheels for adjusting applying amount
impl<'a> Widget for WheelSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let wheel_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.wheels.iter().map(|_| Constraint::Length(21)))
            .split(area);

        for (index, wheel) in self.wheels.iter().enumerate() {
            let is_selected = self.selected_index == index;

            Block::default()
                .title(wheel.label)
                .borders(Borders::ALL)
                .border_type(Rounded)
                .style(wheel.style(is_selected))
                .render(wheel_layout[index], buf);

            let inner_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // gap
                    Constraint::Min(1),    // wheel area
                    Constraint::Length(1), // gap
                    Constraint::Length(1), // values
                ])
                .split(wheel_layout[index].inner(Margin::new(2, 1)));

            Text::from(format!("x:{:.1}, y:{:.1}", wheel.x, wheel.y))
                .style(wheel.style(is_selected))
                .alignment(Alignment::Center)
                .render(inner_layout[3], buf);
            let inner_area = inner_layout[1];

            let center_x = inner_area.x + inner_area.width / 2;
            let center_y = inner_area.y + inner_area.height / 2;

            // Draw the grid and clear the background
            for y in inner_area.top()..inner_area.bottom() {
                let is_top = y == inner_area.top();
                let is_bottom = y == inner_area.bottom().saturating_sub(1);

                for x in inner_area.left()..inner_area.right() {
                    let is_left =
                        x == inner_area.left().saturating_add(1) || x == inner_area.left();
                    let is_right = x == inner_area.right().saturating_sub(2)
                        || x == inner_area.right().saturating_sub(1);

                    let is_center_x = x == center_x;
                    let is_center_y = y == center_y;

                    let symbol = if is_center_x && is_center_y {
                        "┼"
                    } else if is_center_x {
                        "│"
                    } else if is_center_y {
                        "─"
                    } else {
                        " "
                    };

                    let color = match (is_top, is_bottom, is_left, is_right) {
                        (true, _, _, _) => Color::Yellow,
                        (_, true, _, _) => Color::Cyan,
                        (_, _, true, _) => Color::Green,
                        (_, _, _, true) => Color::Magenta,
                        _ => Color::DarkGray,
                    };
                    buf.set_string(x, y, symbol, Style::default().fg(color));
                }
            }

            // Calculate cursor position
            // wheel.x and wheel.y are expected to be between -1.0 and 1.0
            let max_x = (inner_area.width.saturating_sub(1)) as f64 / 2.0;
            let max_y = (inner_area.height.saturating_sub(1)) as f64 / 2.0;

            let cursor_x = (center_x as f64 + wheel.x * max_x).round() as u16;
            // Subtract for Y because terminal coordinates go down, but math goes up
            let cursor_y = (center_y as f64 - wheel.y * max_y).round() as u16;

            let cursor_x = cursor_x.clamp(inner_area.left(), inner_area.right().saturating_sub(1));
            let cursor_y = cursor_y.clamp(inner_area.top(), inner_area.bottom().saturating_sub(1));

            // Draw the cursor
            buf.set_string(cursor_x, cursor_y, "+", wheel.style(is_selected));
        }
    }
}
