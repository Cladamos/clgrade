use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Borders, Widget};

use crate::ui::{CenterOpts, centered_rect, focused_style};

#[derive(Debug)]
pub enum ColorEffects {
    WhiteBalance,
    Exposure,
    Contrast,
    LiftGammaGain,
    Hue,
    Saturation,
}
impl ColorEffects {
    pub fn default() -> Vec<ColorEffects> {
        vec![
            ColorEffects::WhiteBalance,
            ColorEffects::Exposure,
            ColorEffects::Contrast,
            ColorEffects::LiftGammaGain,
            ColorEffects::Hue,
            ColorEffects::Saturation,
        ]
    }

    pub fn get_short_name(&self) -> &str {
        match self {
            ColorEffects::WhiteBalance => "WB",
            ColorEffects::Exposure => "EXP",
            ColorEffects::Contrast => "CONT",
            ColorEffects::LiftGammaGain => "WHEELS",
            ColorEffects::Hue => "HUE",
            ColorEffects::Saturation => "SAT",
        }
    }
}

pub struct PipelineSection<'a> {
    pipeline: &'a Vec<ColorEffects>,
    selected_index: usize,
}

impl<'a> PipelineSection<'a> {
    pub const PIPELINE_HEIGHT: u16 = 5;
    pub const PIPE_WIDTH: u16 = 2;
    pub const BOX_WIDTH: u16 = 10;

    pub fn new(pipeline: &'a Vec<ColorEffects>, selected_index: usize) -> Self {
        PipelineSection {
            pipeline,
            selected_index,
        }
    }
}

impl<'a> Widget for PipelineSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints = self
            .pipeline
            .iter()
            .flat_map(|_| {
                vec![
                    Constraint::Length(PipelineSection::PIPE_WIDTH),
                    Constraint::Length(PipelineSection::BOX_WIDTH),
                ]
                .into_iter()
            })
            .chain(vec![Constraint::Length(PipelineSection::PIPE_WIDTH)]);

        let pipeline_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let pipe_center_opts = CenterOpts {
            width: Self::PIPE_WIDTH,
            height: 1,
            margin: 0,
        };

        for (i, effect) in self.pipeline.iter().enumerate() {
            let is_focused = self.selected_index == i;
            let style = focused_style(is_focused);
            let layout_index = i * 2;
            Line::from("==").style(style).render(
                centered_rect(pipe_center_opts, pipeline_layout[layout_index]),
                buf,
            );
            Block::default()
                .borders(Borders::ALL)
                .border_type(Rounded)
                .style(style)
                .render(pipeline_layout[layout_index + 1], buf);

            let name = effect.get_short_name();
            Text::from(name).render(
                centered_rect(
                    super::CenterOpts {
                        width: name.len() as u16,
                        height: 1,
                        margin: 0,
                    },
                    pipeline_layout[layout_index + 1],
                ),
                buf,
            );

            if i == self.pipeline.len() - 1 {
                Line::from("=>").style(style).render(
                    centered_rect(pipe_center_opts, pipeline_layout[layout_index + 2]),
                    buf,
                );
            }
        }
    }
}
