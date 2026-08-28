use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType::Rounded, Borders, Widget},
};
use tui_slider::{Slider, SliderOrientation, SliderState, style::SliderStyle};

use crate::app::AppLayout;

pub struct SliderData {
    pub label: &'static str,
    pub state: SliderState,
    pub step: f64,
    pub default_value: f64,
}

impl SliderData {
    pub fn style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    }

    pub fn color(&self, selected: bool) -> Color {
        if selected { Color::Blue } else { Color::Gray }
    }
}

pub fn default_sliders() -> Vec<SliderData> {
    vec![
        SliderData {
            label: "Temp",
            state: SliderState::new(0.0, -100.0, 100.0),
            step: 1.0,
            default_value: 0.0,
        },
        SliderData {
            label: "Tint",
            state: SliderState::new(0.0, -100.0, 100.0),
            step: 1.0,
            default_value: 0.0,
        },
        SliderData {
            label: "Exp",
            state: SliderState::new(0.0, -3.0, 3.0),
            step: 0.05,
            default_value: 0.0,
        },
        SliderData {
            label: "Cont",
            state: SliderState::new(0.0, -100.0, 100.0),
            step: 1.0,
            default_value: 0.0,
        },
        SliderData {
            label: "Sat",
            state: SliderState::new(1.0, 0.0, 2.0),
            step: 0.05,
            default_value: 1.0,
        },
        SliderData {
            label: "Hue",
            state: SliderState::new(0.0, -180.0, 180.0),
            step: 2.0,
            default_value: 0.0,
        },
    ]
}

pub fn lum_slider() -> SliderData {
    SliderData {
        label: "Lum",
        state: SliderState::new(0.0, -100.0, 100.0),
        step: 1.0,
        default_value: 0.0,
    }
}

pub struct SliderSection<'a> {
    sliders: &'a [SliderData],
    selected_index: usize,
    app_layout: AppLayout,
}

impl<'a> SliderSection<'a> {
    // Horizontal layout
    pub const PANEL_WIDTH: u16 = 21;
    pub const PANEL_HEIGHT: u16 = 15;
    // Vertical layout
    pub const ITEM_WIDTH: u16 = 11;
    pub const ITEM_HEIGHT: u16 = 5;

    pub fn new(sliders: &'a [SliderData], selected_index: usize, app_layout: AppLayout) -> Self {
        Self {
            sliders,
            selected_index,
            app_layout,
        }
    }
}

impl<'a> Widget for SliderSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let slider_layout = if self.app_layout == AppLayout::Horizontal {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    self.sliders
                        .iter()
                        .map(|_| Constraint::Length(Self::ITEM_HEIGHT)),
                )
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    self.sliders
                        .iter()
                        .map(|_| Constraint::Length(Self::ITEM_WIDTH)),
                )
                .split(area)
        };

        for (index, slider) in self.sliders.iter().enumerate() {
            let is_selected = index == self.selected_index;

            if self.app_layout == AppLayout::Horizontal {
                Block::default()
                    .title(format!("{}: {:.1}", slider.label, slider.state.value()))
                    .borders(Borders::ALL)
                    .border_type(Rounded)
                    .border_style(slider.style(is_selected))
                    .render(slider_layout[index], buf);

                let slider_style = SliderStyle::horizontal();
                Slider::from_state(&slider.state)
                    .orientation(SliderOrientation::Horizontal)
                    .filled_symbol(slider_style.filled_symbol)
                    .handle_symbol(slider_style.handle_symbol)
                    .empty_symbol(slider_style.empty_symbol)
                    .filled_color(slider.color(is_selected))
                    .handle_color(slider.color(is_selected))
                    .empty_color(Color::DarkGray)
                    .render(slider_layout[index].inner(Margin::new(2, 1)), buf);
            } else {
                Block::default()
                    .title(slider.label)
                    .borders(Borders::ALL)
                    .border_type(Rounded)
                    .border_style(slider.style(is_selected))
                    .render(slider_layout[index], buf);
                let inner_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(1), // gap
                        Constraint::Min(1),    // slider area
                        Constraint::Length(1), // gap
                        Constraint::Length(1), // value
                    ])
                    .split(slider_layout[index].inner(Margin::new(0, 1)));

                Slider::from_state(&slider.state)
                    .orientation(SliderOrientation::Vertical)
                    .filled_symbol(SliderStyle::vertical().filled_symbol)
                    .handle_symbol(SliderStyle::vertical().handle_symbol)
                    .empty_symbol(SliderStyle::vertical().empty_symbol)
                    .filled_color(slider.color(is_selected))
                    .handle_color(slider.color(is_selected))
                    .empty_color(Color::DarkGray)
                    .render(inner_layout[1], buf);
                Text::from(format!("{:.2}\n", slider.state.value()))
                    .style(slider.style(is_selected))
                    .alignment(Alignment::Center)
                    .render(inner_layout[3], buf);
            };
        }
    }
}
