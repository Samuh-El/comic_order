use iced::widget::{button, column, container, row, text, text_input, Space, image};
use iced::{Alignment, Element, Length, Theme};

use crate::Message;

#[derive(Debug, Clone)]
pub struct CollectionForm {
    pub id: i64,
    pub name: String,
    pub icon_data: Option<Vec<u8>>,
}

impl CollectionForm {
    pub fn new(id: i64, name: &str, icon_data: Option<Vec<u8>>) -> Self {
        Self {
            id,
            name: name.to_string(),
            icon_data,
        }
    }
}

pub fn view<'a>(form: &'a CollectionForm) -> Element<'a, Message> {
    let icon_display = if let Some(data) = &form.icon_data {
        let handle = image::Handle::from_bytes(data.clone());
        container(image(handle).width(120).height(120))
    } else {
        container(
            text("Sin Imagen (1:1)")
                .size(12)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .width(120)
        .height(120)
        .center_x(120)
        .center_y(120)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.2))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    };

    let overlay = container(
        container(
            column![
                // Header
                row![
                    text("Editar Colección").size(20).color(iced::Color::WHITE),
                    Space::with_width(Length::Fill),
                    button(text("✕").size(16))
                        .padding(4)
                        .on_press(Message::CloseCollectionEditor)
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                            text_color: iced::Color::from_rgb(0.6, 0.6, 0.6),
                            ..Default::default()
                        }),
                ]
                .align_y(Alignment::Center),
                
                // Name field
                column![
                    text("Nombre").size(13).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text_input("Nombre de la colección", &form.name)
                        .on_input(Message::CollectionEditorNameChanged)
                        .padding(10)
                        .size(14),
                ]
                .spacing(4),
                
                // Icon field
                column![
                    text("Imagen de Portada (Remote)").size(13).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    row![
                        icon_display,
                        column![
                            button(text("Seleccionar Imagen").size(13))
                                .padding([8, 16])
                                .on_press(Message::SelectCollectionIcon)
                                .style(|_theme: &Theme, _status| button::Style {
                                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                                        0.25, 0.25, 0.35,
                                    ))),
                                    text_color: iced::Color::WHITE,
                                    border: iced::Border {
                                        radius: 8.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            button(text("Quitar Imagen").size(12))
                                .padding([4, 8])
                                .on_press(Message::RemoveCollectionIcon)
                                .style(|_theme: &Theme, _status| button::Style {
                                    background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                                    text_color: iced::Color::from_rgb(0.6, 0.3, 0.3),
                                    ..Default::default()
                                }),
                        ]
                        .spacing(10)
                    ]
                    .spacing(20)
                    .align_y(Alignment::Center),
                ]
                .spacing(8),
                
                // Buttons
                row![
                    Space::with_width(Length::Fill),
                    button(text("Cancelar").size(13))
                        .padding([8, 16])
                        .on_press(Message::CloseCollectionEditor)
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
                        .on_press(Message::SaveCollectionEditor)
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
            .spacing(20)
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
