use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::Message;

pub fn view<'a>(
    page_handle: Option<&'a iced::widget::image::Handle>,
    current_page: usize,
    total_pages: usize,
    comic_title: &'a str,
    is_loading: bool,
) -> Element<'a, Message> {
    let top_bar = container(
        row![
            button(
                row![text("←").size(18), text("Volver").size(14)]
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .padding([8, 14])
            .on_press(Message::CloseReader)
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
            Space::with_width(Length::Fill),
            text(comic_title).size(14).color(iced::Color::WHITE),
            Space::with_width(Length::Fill),
            text(format!("{} / {}", current_page + 1, total_pages)).size(14).color(iced::Color::from_rgb(0.8, 0.8, 0.8)),
        ]
        .align_y(Alignment::Center)
        .spacing(10),
    )
    .padding([8, 20])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.06, 0.06, 0.10,
        ))),
        ..Default::default()
    });

    let page_display = if is_loading {
        container(
            column![
                text("🔄").size(48),
                text("Cargando...").size(16),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            ..Default::default()
        })
    } else if let Some(handle) = page_handle {
        container(
            image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            ..Default::default()
        })
    } else {
        container(
            text("Sin pagina").size(18),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            ..Default::default()
        })
    };

    let nav_bar = container(
        row![
            button(text("◀ Anterior").size(14))
                .padding([10, 20])
                .on_press_maybe(if current_page > 0 {
                    Some(Message::PrevPage)
                } else {
                    None
                })
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.15, 0.15, 0.25,
                    ))),
                    text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(Length::Fill),
            text("Flechas: ← / →").size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_width(Length::Fill),
            button(text("Siguiente ▶").size(14))
                .padding([10, 20])
                .on_press_maybe(if current_page < total_pages.saturating_sub(1) {
                    Some(Message::NextPage)
                } else {
                    None
                })
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.15, 0.15, 0.25,
                    ))),
                    text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(10),
    )
    .padding([10, 20])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.06, 0.06, 0.10,
        ))),
        ..Default::default()
    });

    column![top_bar, page_display, nav_bar]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}
