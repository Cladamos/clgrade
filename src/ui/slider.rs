use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType::Rounded, Borders, Widget},
};
use tui_slider::{Slider, SliderOrientation, SliderState, style::SliderStyle};

pub struct SliderData {
    pub label: &'static str,
    pub state: SliderState,
    pub step: f64,
    pub slider_style: SliderStyle,
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
            slider_style: SliderStyle::vertical(),
            default_value: 0.0,
        },
        SliderData {
            label: "Tint",
            state: SliderState::new(0.0, -100.0, 100.0),
            step: 1.0,
            slider_style: SliderStyle::vertical(),
            default_value: 0.0,
        },
        SliderData {
            label: "Exp",
            state: SliderState::new(0.0, -3.0, 3.0),
            step: 0.05,
            slider_style: SliderStyle::vertical(),
            default_value: 0.0,
        },
        SliderData {
            label: "Cont",
            state: SliderState::new(0.0, -100.0, 100.0),
            step: 1.0,
            slider_style: SliderStyle::vertical(),
            default_value: 0.0,
        },
        SliderData {
            label: "Sat",
            state: SliderState::new(1.0, 0.0, 2.0),
            step: 0.05,
            slider_style: SliderStyle::vertical(),
            default_value: 1.0,
        },
        SliderData {
            label: "Hue",
            state: SliderState::new(0.0, -180.0, 180.0),
            step: 2.0,
            slider_style: SliderStyle::vertical(),
            default_value: 0.0,
        },
    ]
}

pub fn lum_slider() -> SliderData {
    SliderData {
        label: "Lum",
        state: SliderState::new(0.0, -100.0, 100.0),
        step: 1.0,
        slider_style: SliderStyle::horizontal(),
        default_value: 0.0,
    }
}

pub struct SliderSection<'a> {
    sliders: &'a [SliderData],
    selected_index: usize,
}

impl<'a> SliderSection<'a> {
    pub fn new(sliders: &'a [SliderData], selected_index: usize) -> Self {
        Self {
            sliders,
            selected_index,
        }
    }
}

//TODO: When area is small the value is not visible fix it
impl<'a> Widget for SliderSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let slider_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.sliders.iter().map(|_| Constraint::Length(15)))
            .split(if area.height > 15 {
                Rect { height: 15, ..area }
            } else {
                area
            });

        for (index, slider) in self.sliders.iter().enumerate() {
            let is_selected = index == self.selected_index;

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
                .filled_symbol(slider.slider_style.filled_symbol)
                .handle_symbol(slider.slider_style.handle_symbol)
                .empty_symbol(slider.slider_style.empty_symbol)
                .filled_color(slider.color(is_selected))
                .handle_color(slider.color(is_selected))
                .empty_color(Color::DarkGray)
                .render(inner_layout[1], buf);
            Text::from(format!("{:.2}\n", slider.state.value()))
                .style(slider.style(is_selected))
                .alignment(Alignment::Center)
                .render(inner_layout[3], buf);
        }
    }
}
