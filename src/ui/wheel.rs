use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};
use tui_slider::style::SliderStyle;
use tui_slider::{Slider, SliderOrientation};

use crate::app::AppLayout;
use crate::ui::slider::{SliderData, lum_slider};
use crate::ui::{focused_color, focused_style};

#[derive(PartialEq)]
pub enum SelectedPart {
    Wheel,
    LumSlider,
}
pub struct WheelData {
    pub label: &'static str,
    pub x: f32,
    pub y: f32,
    pub focused_part: SelectedPart,
    pub lum: SliderData,
}

pub struct WheelSection<'a> {
    wheels: &'a [WheelData],
    selected_index: usize,
    app_layout: AppLayout,
}

pub fn default_wheels() -> Vec<WheelData> {
    vec![
        WheelData {
            label: "Lift (Shadows)",
            x: 0.0,
            y: 0.0,
            focused_part: SelectedPart::Wheel,
            lum: lum_slider(),
        },
        WheelData {
            label: "Gamma (Midtones)",
            x: 0.0,
            y: 0.0,
            focused_part: SelectedPart::Wheel,
            lum: lum_slider(),
        },
        WheelData {
            label: "Gain (Highlights)",
            x: 0.0,
            y: 0.0,
            focused_part: SelectedPart::Wheel,
            lum: lum_slider(),
        },
    ]
}

impl<'a> WheelSection<'a> {
    pub const WHEEL_WIDTH: u16 = 21;
    pub const WHEEL_HEIGHT: u16 = 16;
    pub const LUM_AREA_OFFSET: u16 = 4;
    pub const LUM_SLIDER_HEIGHT: u16 = 3;

    pub fn new(wheels: &'a [WheelData], selected_index: usize, app_layout: AppLayout) -> Self {
        Self {
            wheels,
            selected_index,
            app_layout,
        }
    }
    pub fn row_width(wheels: &Vec<WheelData>) -> u16 {
        wheels.len() as u16 * Self::WHEEL_WIDTH
    }

    pub fn col_height(wheels: &Vec<WheelData>) -> u16 {
        wheels.len() as u16 * Self::WHEEL_HEIGHT
    }
}

impl<'a> Widget for WheelSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let direction = match self.app_layout {
            AppLayout::Horizontal => Direction::Vertical,
            AppLayout::Vertical => Direction::Horizontal,
        };
        let wheel_layout = Layout::default()
            .direction(direction)
            .constraints(
                self.wheels
                    .iter()
                    .map(|_| Constraint::Length(Self::WHEEL_WIDTH)),
            )
            .split(area);

        for (index, wheel) in self.wheels.iter().enumerate() {
            let is_wheel_focused =
                self.selected_index == index && matches!(wheel.focused_part, SelectedPart::Wheel);
            let is_lum_focused = self.selected_index == index
                && matches!(wheel.focused_part, SelectedPart::LumSlider);

            let wheel_area = Rect {
                x: wheel_layout[index].x,
                y: wheel_layout[index].y,
                width: wheel_layout[index].width,
                height: wheel_layout[index]
                    .height
                    .saturating_sub(Self::LUM_AREA_OFFSET),
            };
            let style = focused_style(is_wheel_focused);
            Block::default()
                .title(wheel.label)
                .borders(Borders::ALL)
                .border_type(Rounded)
                .style(style)
                .render(wheel_area, buf);

            let inner_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // gap
                    Constraint::Min(1),    // wheel area
                    Constraint::Length(1), // gap
                    Constraint::Length(1), // values
                ])
                .split(wheel_area.inner(Margin::new(2, 1)));

            Text::from(format!("x:{:.1}, y:{:.1}", wheel.x, wheel.y))
                .style(style)
                .alignment(Alignment::Center)
                .render(inner_layout[3], buf);
            let inner_area = inner_layout[1];

            let center_x = inner_area.x + inner_area.width / 2;
            let center_y = inner_area.y + inner_area.height / 2;

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
                        (_, true, _, _) => Color::Blue,
                        (_, _, true, _) => Color::Green,
                        (_, _, _, true) => Color::Magenta,
                        _ => Color::DarkGray,
                    };
                    buf.set_string(x, y, symbol, Style::default().fg(color));
                }
            }

            // wheel.x and wheel.y are expected to be between -1.0 and 1.0
            let max_x = (inner_area.width.saturating_sub(1)) as f32 / 2.0;
            let max_y = (inner_area.height.saturating_sub(1)) as f32 / 2.0;

            let cursor_x = (center_x as f32 + wheel.x * max_x).round() as u16;
            // Subtract for Y because terminal coordinates go down, but math goes up
            let cursor_y = (center_y as f32 - wheel.y * max_y).round() as u16;

            let cursor_x = cursor_x.clamp(inner_area.left(), inner_area.right().saturating_sub(1));
            let cursor_y = cursor_y.clamp(inner_area.top(), inner_area.bottom().saturating_sub(1));

            buf.set_string(cursor_x, cursor_y, "+", style);

            let style = focused_style(is_lum_focused);
            let color = focused_color(is_lum_focused);
            let slider_area = Rect {
                x: wheel_layout[index].x,
                y: wheel_layout[index]
                    .bottom()
                    .saturating_sub(Self::LUM_AREA_OFFSET),
                width: wheel_layout[index].width,
                height: Self::LUM_SLIDER_HEIGHT,
            };
            let slider = &wheel.lum;
            Block::default()
                .title(format!("{}: {:.1}", slider.label, slider.state.value()))
                .borders(Borders::ALL)
                .border_type(Rounded)
                .border_style(style)
                .render(slider_area, buf);

            let slider_style = SliderStyle::horizontal();
            Slider::from_state(&slider.state)
                .orientation(SliderOrientation::Horizontal)
                .filled_symbol(slider_style.filled_symbol)
                .handle_symbol(slider_style.handle_symbol)
                .empty_symbol(slider_style.empty_symbol)
                .filled_color(color)
                .handle_color(color)
                .empty_color(Color::DarkGray)
                .render(slider_area.inner(Margin::new(2, 0)), buf);
        }
    }
}
