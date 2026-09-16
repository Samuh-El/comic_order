use iced::widget::{button, column, container, row, scrollable, text, text_input, image, Space, mouse_area, svg};
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
) -> Element<'a, Message> {
    
    let conn_icon = include_bytes!("../../assets/reading-icon.png");
    let conn_handle = image::Handle::from_bytes(conn_icon.as_slice());

    let coll_icon = include_bytes!("../../assets/slideshow-icon.png");
    let coll_handle = image::Handle::from_bytes(coll_icon.as_slice());

    let stop_icon = include_bytes!("../../assets/pause-round-icon.png");
    let stop_handle = image::Handle::from_bytes(stop_icon.as_slice());

    let layer_icon = include_bytes!("../../assets/layer-icon.png");
    let layer_handle = image::Handle::from_bytes(layer_icon.as_slice());


    let mut list = column![].spacing(2).padding(5);

    for collection in collections {
        let cid = collection.id;
        let is_selected = selected_id == Some(cid);

        let btn_content = row![
            image(layer_handle.clone()).width(16).height(16),
            text(collection.name.clone()).size(14).color(iced::Color::WHITE),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let btn = button(btn_content)
            .width(Length::Fill)
            .padding(10)
            .on_press(Message::SelectCollection(cid))
            .style(move |_theme: &Theme, status| {
                let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
                
                if is_selected {
                    button::Style {
                        background: Some(iced::Background::Color(if is_hovered {
                            iced::Color::from_rgb(0.25, 0.25, 0.45)
                        } else {
                            iced::Color::from_rgb(0.15, 0.15, 0.30)
                        })),
                        text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: if is_hovered {
                            Some(iced::Background::Color(iced::Color::from_rgba(0.2, 0.2, 0.2, 0.4)))
                        } else {
                            Some(iced::Background::Color(iced::Color::TRANSPARENT))
                        },
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }
            });

        let btn = mouse_area(btn)
            .on_right_press(Message::ToggleCollectionMenu(cid));

        list = list.push(container(btn).width(Length::Fill));

        // Show context menu dropdown if this collection's menu is open
        if context_menu_id == Some(cid) {
            let menu = container(
                column![
                    button(
                        row![
                            svg(svg::Handle::from_memory(include_bytes!("../../assets/edit-svgrepo-com.svg").as_slice()))
                                .width(14)
                                .height(14)
                                .style(|_theme: &Theme, _status| svg::Style {
                                    color: Some(iced::Color::from_rgb(0.85, 0.85, 0.85)),
                                }),
                            text("Editar").size(12)
                        ].spacing(8)
                    )
                    .width(Length::Fill)
                    .padding([6, 10])
                    .on_press(Message::EditCollection(cid))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                        text_color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                        border: iced::Border { radius: 4.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                    button(
                        row![
                            svg(svg::Handle::from_memory(include_bytes!("../../assets/delete-1487-svgrepo-com.svg").as_slice()))
                                .width(14)
                                .height(14)
                                .style(|_theme: &Theme, _status| svg::Style {
                                    color: Some(iced::Color::from_rgb(0.91, 0.27, 0.37)),
                                }),
                            text("Eliminar").size(12)
                        ].spacing(8)
                    )
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
            row![
                image(coll_handle).width(18).height(18),
                text("Nueva Coleccion").size(13)
            ].spacing(8)
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
    let qr_label = if server_running {
        "[ON] Servidor Activo"
    } else {
        "Compartir QR"
    };
    let qr_btn = button(
        row![
            image(conn_handle).width(24).height(24),
            text(qr_label).size(13)
        ]
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

    let mut share_section = column![qr_btn].spacing(8);

    if server_running {
        let stop_btn = button(
            row![
                image(stop_handle.clone()).width(14).height(14),
                text("Detener Compartir").size(13)
            ]
            .spacing(10)
            .width(Length::Fill)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding(12)
        .on_press(Message::StopServer)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            text_color: iced::Color::from_rgb(0.91, 0.27, 0.37),
            border: iced::Border {
                color: iced::Color::from_rgb(0.91, 0.27, 0.37),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });
        share_section = share_section.push(stop_btn);
    }

    // Manage Trusted Devices Button
    let manage_devices_btn = button(
        row![
            svg(svg::Handle::from_memory(include_bytes!("../../assets/devices-svgrepo-com.svg").as_slice()))
                .width(18)
                .height(18)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.8)),
                }),
            text("Dispositivos Recurrentes").size(13)
        ]
        .spacing(10)
        .width(Length::Fill)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(12)
    .on_press(Message::ManageTrustedDevices)
    .style(|_theme: &Theme, _status| button::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.20))),
        text_color: iced::Color::from_rgb(0.7, 0.7, 0.8),
        border: iced::Border {
            color: iced::Color::from_rgb(0.3, 0.3, 0.4),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    share_section = share_section.push(manage_devices_btn);

    container(
        column![
            scrollable(list).height(Length::Fill),
            Space::with_height(Length::Fill),
            bottom_section,
            container(share_section).padding(10),
        ]
        .height(Length::Fill),
    )
    .width(200)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.15, 0.15, 0.18, 0.95,
        ))),
        ..Default::default()
    })
    .into()
}
