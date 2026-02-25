use iced::widget::{button, column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::db::Comic;
use crate::Message;

pub fn view<'a>(
    comics: &'a [Comic],
    collection_name: &'a str,
    comic_handles: &'a std::collections::HashMap<i64, iced::widget::image::Handle>,
    is_loading: bool,
) -> Element<'a, Message> {
    let header = container(
        row![
            column![
                text(collection_name).size(24),
                text(format!("{} comics", comics.len())).size(13),
            ]
            .spacing(4),
            Space::with_width(Length::Fill),
            button(
                row![text("📂").size(14), text("Añadir Carpeta").size(13)]
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .on_press(Message::AddPath)
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
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding(20);

    if is_loading {
        let spinner = container(
            column![
                text("🔄").size(48),
                text("Escaneando...").size(16),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        return column![header, spinner].into();
    }

    if comics.is_empty() {
        let empty = container(
            column![
                text("📚").size(64),
                text("No hay comics en esta colección").size(16),
                text("Haz clic en 'Añadir Carpeta' para agregar comics").size(13),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        return column![header, empty].into();
    }

    // Build comic cards in a wrapped grid
    let cards_per_row = 5;
    let mut grid = column![].spacing(15).padding(20);
    let mut current_row = row![].spacing(15);
    let mut count = 0;

    for comic in comics {
        let mut card_content = column![].spacing(4);

        // Cover image or placeholder
        if let Some(handle) = comic_handles.get(&comic.id) {
            card_content = card_content.push(
                container(image(handle.clone()).width(160).height(240))
                    .style(|_theme: &Theme| container::Style {
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        } else {
            card_content = card_content.push(
                container(
                    text("[portada]").size(14),
                )
                .width(160)
                .height(240)
                .center_x(160)
                .center_y(240)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.15, 0.15, 0.25,
                    ))),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        // Title
        let title_text = if comic.title.len() > 22 {
            format!("{}...", &comic.title[..20])
        } else {
            comic.title.clone()
        };
        card_content = card_content.push(text(title_text).size(12));

        // Meta info
        let mut meta = String::new();
        if let Some(year) = comic.year {
            meta.push_str(&year.to_string());
        }
        if let Some(num) = comic.issue_number {
            if !meta.is_empty() {
                meta.push_str(" - ");
            }
            meta.push_str(&format!("#{}", num));
        }
        if meta.is_empty() {
            meta = comic.file_type.to_uppercase();
        }
        card_content = card_content.push(text(meta).size(11));

        let comic_id = comic.id;

        let read_btn = button(card_content)
            .padding(0)
            .on_press(Message::OpenComic(comic_id))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                ..Default::default()
            });

        let edit_btn = button(text("[editar]").size(11))
            .padding(4)
            .on_press(Message::EditComic(comic_id))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
                ..Default::default()
            });

        let card = container(
            column![read_btn, edit_btn].spacing(2).align_x(Alignment::Center),
        )
        .padding(10)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.1, 0.1, 0.18,
            ))),
            border: iced::Border {
                radius: 12.0.into(),
                color: iced::Color::from_rgb(0.15, 0.15, 0.25),
                width: 1.0,
            },
            ..Default::default()
        });

        current_row = current_row.push(card);
        count += 1;

        if count % cards_per_row == 0 {
            grid = grid.push(current_row);
            current_row = row![].spacing(15);
        }
    }

    if count % cards_per_row != 0 {
        grid = grid.push(current_row);
    }

    column![header, scrollable(grid).height(Length::Fill)]
        .height(Length::Fill)
        .into()
}
