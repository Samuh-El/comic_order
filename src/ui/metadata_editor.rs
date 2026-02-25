use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::Message;

#[derive(Debug, Clone)]
pub struct MetadataForm {
    pub comic_id: i64,
    pub title: String,
    pub year: String,
    pub issue_number: String,
    pub saga: String,
}

impl MetadataForm {
    pub fn new(comic_id: i64, title: &str, year: Option<i32>, issue_number: Option<i32>, saga: Option<&str>) -> Self {
        Self {
            comic_id,
            title: title.to_string(),
            year: year.map(|y| y.to_string()).unwrap_or_default(),
            issue_number: issue_number.map(|n| n.to_string()).unwrap_or_default(),
            saga: saga.unwrap_or("").to_string(),
        }
    }
}

pub fn view<'a>(form: &'a MetadataForm) -> Element<'a, Message> {
    let overlay = container(
        container(
            column![
                // Header
                row![
                    text("Editar Comic").size(20),
                    Space::with_width(Length::Fill),
                    button(text("✕").size(16))
                        .padding(4)
                        .on_press(Message::CloseEditor)
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                            text_color: iced::Color::from_rgb(0.6, 0.6, 0.6),
                            ..Default::default()
                        }),
                ]
                .align_y(Alignment::Center),
                
                // Title field
                column![
                    text("Título").size(13),
                    text_input("Título del comic", &form.title)
                        .on_input(Message::EditorTitleChanged)
                        .padding(10)
                        .size(14),
                ]
                .spacing(4),
                
                // Year and Issue row
                row![
                    column![
                        text("Año").size(13),
                        text_input("2024", &form.year)
                            .on_input(Message::EditorYearChanged)
                            .padding(10)
                            .size(14),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    column![
                        text("Número").size(13),
                        text_input("#1", &form.issue_number)
                            .on_input(Message::EditorIssueChanged)
                            .padding(10)
                            .size(14),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                ]
                .spacing(10),
                
                // Saga field
                column![
                    text("Saga").size(13),
                    text_input("Nombre de la saga", &form.saga)
                        .on_input(Message::EditorSagaChanged)
                        .padding(10)
                        .size(14),
                ]
                .spacing(4),
                
                // Buttons
                row![
                    Space::with_width(Length::Fill),
                    button(text("Cancelar").size(13))
                        .padding([8, 16])
                        .on_press(Message::CloseEditor)
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(
                                0.2, 0.2, 0.3,
                            ))),
                            text_color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                            border: iced::Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    button(text("Guardar").size(13))
                        .padding([8, 16])
                        .on_press(Message::SaveMetadata)
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(
                                0.91, 0.27, 0.37,
                            ))),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(10),
            ]
            .spacing(15)
            .padding(25),
        )
        .width(450)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.1, 0.1, 0.18,
            ))),
            border: iced::Border {
                radius: 16.0.into(),
                color: iced::Color::from_rgb(0.2, 0.2, 0.3),
                width: 1.0,
            },
            ..Default::default()
        }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.0, 0.0, 0.0, 0.7,
        ))),
        ..Default::default()
    });

    overlay.into()
}
