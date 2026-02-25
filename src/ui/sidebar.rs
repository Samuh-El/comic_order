use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::db::Collection;
use crate::Message;

pub fn view<'a>(
    collections: &'a [Collection],
    selected_id: Option<i64>,
    new_collection_name: &'a str,
    show_new_input: bool,
    server_running: bool,
    context_menu_id: Option<i64>,
    renaming_id: Option<i64>,
    rename_input: &'a str,
) -> Element<'a, Message> {
    let title = container(
        text("📚 COMIC")
            .size(26),
    )
    .padding(15)
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.91, 0.27, 0.37,
        ))),
        text_color: Some(iced::Color::WHITE),
        ..Default::default()
    });

    let mut list = column![].spacing(2).padding(5);

    for collection in collections {
        let cid = collection.id;
        let is_selected = selected_id == Some(cid);

        // Check if this collection is being renamed
        if renaming_id == Some(cid) {
            let rename_row = container(
                column![
                    text_input("Nuevo nombre...", rename_input)
                        .on_input(Message::RenameInputChanged)
                        .on_submit(Message::ConfirmRename)
                        .padding(8)
                        .size(13),
                    row![
                        button(text("OK").size(12))
                            .padding([4, 10])
                            .on_press(Message::ConfirmRename)
                            .style(|_theme: &Theme, _status| button::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgb(
                                    0.13, 0.55, 0.33,
                                ))),
                                text_color: iced::Color::WHITE,
                                border: iced::Border { radius: 6.0.into(), ..Default::default() },
                                ..Default::default()
                            }),
                        button(text("X").size(12))
                            .padding([4, 10])
                            .on_press(Message::ToggleCollectionMenu(cid))
                            .style(|_theme: &Theme, _status| button::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgb(
                                    0.5, 0.2, 0.2,
                                ))),
                                text_color: iced::Color::WHITE,
                                border: iced::Border { radius: 6.0.into(), ..Default::default() },
                                ..Default::default()
                            }),
                    ]
                    .spacing(5)
                ]
                .spacing(5)
            )
            .padding(8)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.12, 0.12, 0.22,
                ))),
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                    width: 1.0,
                },
                ..Default::default()
            });

            list = list.push(rename_row);
            continue;
        }

        let label = if is_selected {
            format!("📖 {}", &collection.name)
        } else {
            format!("📁 {}", &collection.name)
        };

        let btn = button(
            text(label).size(14),
        )
        .width(Length::Fill)
        .padding(10)
        .on_press(Message::SelectCollection(cid))
        .style(move |_theme: &Theme, _status| {
            if is_selected {
                button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.15, 0.15, 0.30,
                    ))),
                    text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            } else {
                button::Style {
                    background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                    text_color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        });

        // Context menu button (three dots)
        let menu_btn = button(text("⋮").size(14))
            .padding([5, 8])
            .on_press(Message::ToggleCollectionMenu(cid))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                text_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
                ..Default::default()
            });

        let collection_row = row![btn, menu_btn]
            .align_y(Alignment::Center)
            .spacing(0);

        list = list.push(collection_row);

        // Show context menu dropdown if this collection's menu is open
        if context_menu_id == Some(cid) {
            let menu = container(
                column![
                    button(row![text("✏️").size(12), text("Renombrar").size(12)].spacing(6))
                    .width(Length::Fill)
                    .padding([6, 10])
                    .on_press(Message::StartRenameCollection(cid))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                        text_color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                    button(row![text("🗑️").size(12), text("Eliminar").size(12)].spacing(6))
                    .width(Length::Fill)
                    .padding([6, 10])
                    .on_press(Message::DeleteCollection(cid))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                        text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                ]
                .spacing(2),
            )
            .padding(6)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.14, 0.14, 0.24,
                ))),
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color::from_rgb(0.25, 0.25, 0.35),
                    width: 1.0,
                },
                ..Default::default()
            });

            list = list.push(menu);
        }
    }

    let mut bottom_section = column![].spacing(8).padding(10);

    if show_new_input {
        let input = text_input("Nombre de coleccion...", new_collection_name)
            .on_input(Message::NewCollectionNameChanged)
            .on_submit(Message::CreateCollection)
            .padding(8)
            .size(14);

        let create_btn = button(text("✅ Crear").size(13))
            .padding([6, 12])
            .on_press(Message::CreateCollection)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.91, 0.27, 0.37,
                ))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        bottom_section = bottom_section
            .push(input)
            .push(create_btn);
    } else {
        let add_btn = button(
            text("📁 + Nueva Coleccion").size(13),
        )
        .width(Length::Fill)
        .padding(10)
        .on_press(Message::ToggleNewCollection)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.12, 0.12, 0.20,
            ))),
            text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
            border: iced::Border {
                color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

        bottom_section = bottom_section.push(add_btn);
    }

    // QR / Share button - always visible
    let (qr_label, qr_icon) = if server_running {
        ("[ON] Servidor Activo", "🟢")
    } else {
        ("Compartir QR", "📱")
    };
    let qr_btn = button(
        row![text(qr_icon).size(14), text(qr_label).size(13)]
            .spacing(10)
            .width(Length::Fill)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(12)
    .on_press(Message::ToggleServer)
    .style(move |_theme: &Theme, _status| {
        if server_running {
            button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.13, 0.55, 0.33,
                ))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        } else {
            button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.91, 0.27, 0.37,
                ))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    });

    container(
        column![
            title,
            scrollable(list).height(Length::Fill),
            Space::with_height(Length::Fill),
            bottom_section,
            container(qr_btn).padding(10),
        ]
        .height(Length::Fill),
    )
    .width(220)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.08, 0.08, 0.14,
        ))),
        ..Default::default()
    })
    .into()
}
