use std::{collections::HashMap, rc::Rc};

use crate::{config, data, ui};

use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// A reference to the index of all notes
    index: Rc<HashMap<String, data::Note>>,
    /// The internal stats of the displayed note.
    note: data::Note,
    /// The linkage stats of the displayed note
    note_env: Option<data::NoteEnvStatistics>,
    /// The used styling theme
    styles: ui::UiStyles,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: String,
        index: Rc<HashMap<String, data::Note>>,
        config: &config::Config,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            note: index.get(&note_id).cloned().unwrap_or_default(),
            note_env: data::EnvironmentStats::new_with_filters(&index, data::Filter::default())
                .filtered_stats
                .iter()
                .find(|env_stats| env_stats.id == note_id)
                .cloned(),
            index,
            styles: config.get_ui_styles().to_owned(),
        })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        // Generate vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(6),
        ]);

        let [title_area, stats_area, links_area] = vertical.areas(area);

        // Title
        let title = Line::from(vec![
            Span::styled(self.note.name.as_str(), self.styles.title_style),
            Span::styled(" - ", self.styles.text_style),
            Span::styled(
                self.note.path.to_str().unwrap_or_default(),
                self.styles.text_style,
            ),
        ])
        .alignment(Alignment::Center);

        // // Stats Area
        // let stats_rows = [
        //     Row::new(vec![
        //         "Total notes:",
        //         &global_strings[0],
        //         "Total words:",
        //         &global_strings[1],
        //     ]),
        //     Row::new(vec![
        //         "Total unique tags:",
        //         &global_strings[2],
        //         "Total characters:",
        //         &global_strings[3],
        //     ]),
        //     Row::new(vec![
        //         "Total links:",
        //         &global_strings[4],
        //         "Broken links:",
        //         &global_strings[5],
        //     ]),
        // ];

        // let stats = Table::new(stats_rows, stats_widths)
        //     .column_spacing(1)
        //     .block(Block::bordered().title("Statistics".set_style(self.styles.title_style)));

        Widget::render(title, title_area, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}
