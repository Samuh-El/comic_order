//! Barra superior fija unificada con menús temáticos y desplegables.
//!
//! Sustituye la barra lateral anterior y ofrece navegación global hacia:
//! - INICIO (Hero Showcase)
//! - COLECCIONES ▾ (Listado y creación)
//! - CÓMICS ▾ (Re-escaneo y visualización)
//! - SERVIDOR / QR ▾ (Estado de red local y emparejamiento)
//! - Reloj en tiempo real

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme, Vector};

use crate::domain::collection::Collection;

/// Identificador de los menús desplegables disponibles en la barra superior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarMenu {
    Collections,
    Comics,
    Server,
}

/// Mensajes de interacción emitidos por la barra superior.
#[derive(Debug, Clone)]
pub enum TopBarMessage {
    NavigateHome,
    ToggleMenu(TopBarMenu),
    CloseMenu,
    SelectCollection(i64),
    NewCollectionClicked,
    RescanLibrariesClicked,
    ToggleServerClicked,
    OpenQrModal,
    OpenTrustedDevices,
}

/// Renderiza la barra superior fija unificada y el menú desplegable activo si existe.
pub fn view<'a, Message>(
    active_menu: Option<TopBarMenu>,
    collections: &'a [Collection],
    selected_collection_id: Option<i64>,
    server_running: bool,
    server_url: Option<&'a str>,
    current_time: &'a str,
    map_msg: impl Fn(TopBarMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    // Botón INICIO
    let is_home_active = selected_collection_id.is_none();
    let home_btn = button(
        text("INICIO")
            .size(13)
            .font(iced::Font::with_name("Segoe UI Bold"))
            .color(if is_home_active {
                Color::from_rgb(1.0, 0.2, 0.25)
            } else {
                Color::from_rgb(0.85, 0.87, 0.92)
            }),
    )
    .padding(iced::Padding {
        top: 8.0,
        bottom: 8.0,
        left: 14.0,
        right: 14.0,
    })
    .on_press(map_msg(TopBarMessage::NavigateHome))
    .style(move |_theme: &Theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: if is_home_active {
                Some(Background::Color(Color::from_rgba(0.9, 0.1, 0.15, 0.15)))
            } else if is_hovered {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
            } else {
                None
            },
            border: Border {
                color: if is_home_active {
                    Color::from_rgb(0.9, 0.1, 0.15)
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    });

    // Menú desplegable COLECCIONES ▾
    let collections_menu_open = active_menu == Some(TopBarMenu::Collections);
    let collections_btn = create_menu_trigger(
        "COLECCIONES ▾",
        collections_menu_open,
        map_msg(TopBarMessage::ToggleMenu(TopBarMenu::Collections)),
    );

    // Menú desplegable CÓMICS ▾
    let comics_menu_open = active_menu == Some(TopBarMenu::Comics);
    let comics_btn = create_menu_trigger(
        "CÓMICS ▾",
        comics_menu_open,
        map_msg(TopBarMessage::ToggleMenu(TopBarMenu::Comics)),
    );

    // Menú desplegable SERVIDOR / QR ▾
    let server_menu_open = active_menu == Some(TopBarMenu::Server);
    let server_indicator_color = if server_running {
        Color::from_rgb(0.2, 0.85, 0.3)
    } else {
        Color::from_rgb(0.6, 0.6, 0.6)
    };

    let server_btn = button(
        row![
            container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(server_indicator_color)),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(8),
            text("SERVIDOR / QR ▾")
                .size(13)
                .font(iced::Font::with_name("Segoe UI Semibold"))
                .color(Color::from_rgb(0.85, 0.87, 0.92)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(iced::Padding {
        top: 8.0,
        bottom: 8.0,
        left: 14.0,
        right: 14.0,
    })
    .on_press(map_msg(TopBarMessage::ToggleMenu(TopBarMenu::Server)))
    .style(move |_theme: &Theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: if server_menu_open {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)))
            } else if is_hovered {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
            } else {
                None
            },
            border: Border {
                color: if server_menu_open {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    });

    // Reloj a la derecha
    let clock_label = text(current_time)
        .size(13)
        .font(iced::Font::with_name("Segoe UI"))
        .color(Color::from_rgb(0.7, 0.72, 0.78));

    // Barra principal
    let bar_content = row![
        home_btn,
        Space::with_width(6),
        collections_btn,
        Space::with_width(6),
        comics_btn,
        Space::with_width(6),
        server_btn,
        Space::with_width(Length::Fill),
        clock_label,
    ]
    .align_y(Alignment::Center)
    .padding(iced::Padding {
        top: 6.0,
        bottom: 6.0,
        left: 24.0,
        right: 24.0,
    });

    let main_bar = container(bar_content)
        .width(Length::Fill)
        .height(Length::Fixed(52.0))
        .center_y(Length::Fixed(52.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.06, 0.07, 0.10, 0.95))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

    // Dropdown flotante si algún menú está activo
    if let Some(menu) = active_menu {
        let dropdown = render_dropdown_menu(
            menu,
            collections,
            selected_collection_id,
            server_running,
            server_url,
            map_msg,
        );

        column![main_bar, dropdown].into()
    } else {
        main_bar.into()
    }
}

/// Crea el botón disparador de un menú desplegable.
fn create_menu_trigger<'a, Message>(
    label: &'static str,
    is_open: bool,
    msg: Message,
) -> iced::widget::Button<'a, Message>
where
    Message: 'a + Clone,
{
    button(
        text(label)
            .size(13)
            .font(iced::Font::with_name("Segoe UI Semibold"))
            .color(Color::from_rgb(0.85, 0.87, 0.92)),
    )
    .padding(iced::Padding {
        top: 8.0,
        bottom: 8.0,
        left: 14.0,
        right: 14.0,
    })
    .on_press(msg)
    .style(move |_theme: &Theme, status| {
        let is_hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: if is_open {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)))
            } else if is_hovered {
                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
            } else {
                None
            },
            border: Border {
                color: if is_open {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    })
}

/// Renderiza el panel flotante correspondiente al menú desplegado.
fn render_dropdown_menu<'a, Message>(
    menu: TopBarMenu,
    collections: &'a [Collection],
    selected_collection_id: Option<i64>,
    server_running: bool,
    server_url: Option<&'a str>,
    map_msg: impl Fn(TopBarMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let content: Element<'a, Message> = match menu {
        TopBarMenu::Collections => {
            let mut items = column![].spacing(4);

            // Opción para crear nueva colección
            let new_col_btn = button(
                row![
                    text("+").size(16).color(Color::from_rgb(1.0, 0.3, 0.35)),
                    Space::with_width(8),
                    text("Nueva Colección...")
                        .size(13)
                        .font(iced::Font::with_name("Segoe UI Semibold"))
                        .color(Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                bottom: 8.0,
                left: 12.0,
                right: 12.0,
            })
            .on_press(map_msg(TopBarMessage::NewCollectionClicked))
            .style(|_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: if is_hovered {
                        Some(Background::Color(Color::from_rgba(0.9, 0.1, 0.15, 0.25)))
                    } else {
                        None
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            });

            items = items.push(new_col_btn);

            if !collections.is_empty() {
                // Divisor
                items = items.push(
                    container(Space::new(Length::Fill, Length::Fixed(1.0))).style(
                        |_theme: &Theme| container::Style {
                            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
                            ..Default::default()
                        },
                    ),
                );

                for col in collections {
                    let col_id = col.id;
                    let is_current = selected_collection_id == Some(col_id);

                    let item_btn = button(
                        row![
                            text("📁").size(14),
                            Space::with_width(8),
                            text(&col.name)
                                .size(13)
                                .color(if is_current {
                                    Color::from_rgb(1.0, 0.3, 0.35)
                                } else {
                                    Color::from_rgb(0.85, 0.87, 0.92)
                                }),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding(iced::Padding {
                        top: 7.0,
                        bottom: 7.0,
                        left: 12.0,
                        right: 12.0,
                    })
                    .on_press(map_msg(TopBarMessage::SelectCollection(col_id)))
                    .style(move |_theme: &Theme, status| {
                        let is_hovered = matches!(status, button::Status::Hovered);
                        button::Style {
                            background: if is_current {
                                Some(Background::Color(Color::from_rgba(0.9, 0.1, 0.15, 0.15)))
                            } else if is_hovered {
                                Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06)))
                            } else {
                                None
                            },
                            border: Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            text_color: Color::WHITE,
                            ..Default::default()
                        }
                    });

                    items = items.push(item_btn);
                }
            }

            container(items)
                .width(Length::Fixed(240.0))
                .padding(8)
                .into()
        }

        TopBarMenu::Comics => {
            let rescan_btn = button(
                row![
                    text("🔄").size(14),
                    Space::with_width(8),
                    text("Re-escanear Bibliotecas")
                        .size(13)
                        .color(Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                bottom: 8.0,
                left: 12.0,
                right: 12.0,
            })
            .on_press(map_msg(TopBarMessage::RescanLibrariesClicked))
            .style(|_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: if is_hovered {
                        Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                    } else {
                        None
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            });

            container(column![rescan_btn])
                .width(Length::Fixed(220.0))
                .padding(8)
                .into()
        }

        TopBarMenu::Server => {
            let status_text = if server_running {
                "Servidor Activo (Puerto 8080)"
            } else {
                "Servidor Detenido"
            };

            let status_header = row![
                container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0))).style(
                    move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(if server_running {
                            Color::from_rgb(0.2, 0.85, 0.3)
                        } else {
                            Color::from_rgb(0.6, 0.6, 0.6)
                        })),
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
                Space::with_width(8),
                text(status_text)
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.73, 0.80)),
            ]
            .align_y(Alignment::Center);

            let toggle_label = if server_running {
                "Detener Servidor"
            } else {
                "Iniciar Servidor"
            };

            let toggle_btn = button(
                text(toggle_label)
                    .size(13)
                    .font(iced::Font::with_name("Segoe UI Semibold"))
                    .color(Color::WHITE),
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                bottom: 8.0,
                left: 12.0,
                right: 12.0,
            })
            .on_press(map_msg(TopBarMessage::ToggleServerClicked))
            .style(move |_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);
                let bg_color = if server_running {
                    if is_hovered {
                        Color::from_rgb(0.85, 0.15, 0.2)
                    } else {
                        Color::from_rgb(0.7, 0.1, 0.15)
                    }
                } else if is_hovered {
                    Color::from_rgb(0.15, 0.7, 0.3)
                } else {
                    Color::from_rgb(0.1, 0.55, 0.25)
                };

                button::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            });

            let qr_btn = button(
                row![
                    text("📱").size(14),
                    Space::with_width(8),
                    text("Mostrar Código QR")
                        .size(13)
                        .color(Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                bottom: 8.0,
                left: 12.0,
                right: 12.0,
            })
            .on_press(map_msg(TopBarMessage::OpenQrModal))
            .style(|_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: if is_hovered {
                        Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                    } else {
                        None
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            });

            let devices_btn = button(
                row![
                    text("🔒").size(14),
                    Space::with_width(8),
                    text("Dispositivos de Confianza")
                        .size(13)
                        .color(Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                bottom: 8.0,
                left: 12.0,
                right: 12.0,
            })
            .on_press(map_msg(TopBarMessage::OpenTrustedDevices))
            .style(|_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: if is_hovered {
                        Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                    } else {
                        None
                    },
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            });

            let mut server_col = column![
                status_header,
                Space::with_height(6),
                toggle_btn,
                Space::with_height(6),
                qr_btn,
                devices_btn,
            ]
            .spacing(2);

            if let Some(url) = server_url {
                server_col = server_col.push(Space::with_height(4)).push(
                    text(format!("URL: {}", url))
                        .size(11)
                        .color(Color::from_rgb(0.5, 0.75, 1.0)),
                );
            }

            container(server_col)
                .width(Length::Fixed(260.0))
                .padding(12)
                .into()
        }
    };

    container(content)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.08, 0.09, 0.13, 0.96))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 4.0,
            left: match menu {
                TopBarMenu::Collections => 80.0,
                TopBarMenu::Comics => 210.0,
                TopBarMenu::Server => 310.0,
            },
            bottom: 0.0,
            right: 0.0,
        })
        .into()
}
