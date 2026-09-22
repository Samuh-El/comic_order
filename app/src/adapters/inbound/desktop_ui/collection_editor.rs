//! Componente modal para creación, edición y personalización de colecciones.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, image, svg};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme, Vector};

use crate::Message;

#[derive(Debug, Clone)]
pub struct CollectionForm {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub background_image_path: Option<String>,
    pub hero_image_path: Option<String>,
    pub icon_data: Option<Vec<u8>>,
}

impl CollectionForm {
    pub fn new(
        id: i64,
        name: &str,
        description: Option<String>,
        background_image_path: Option<String>,
        hero_image_path: Option<String>,
        icon_data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.unwrap_or_default(),
            background_image_path,
            hero_image_path,
            icon_data,
        }
    }
}

pub fn view<'a>(form: &'a CollectionForm) -> Element<'a, Message> {
    let title_label = if form.id == 0 {
        "Nueva Colección"
    } else {
        "Editar Colección"
    };

    let icon_display = if let Some(data) = &form.icon_data {
        let handle = image::Handle::from_bytes(data.clone());
        container(image(handle).width(90).height(90))
    } else {
        container(
            text("Sin Imagen (1:1)")
                .size(11)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .width(90)
        .height(90)
        .center_x(90)
        .center_y(90)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    };

    // Nombre de archivo o ruta de fondo
    let bg_filename = form
        .background_image_path
        .as_deref()
        .map(|p| std::path::Path::new(p).file_name().and_then(|n| n.to_str()).unwrap_or(p))
        .unwrap_or("Sin fondo panorámico");

    // Nombre de archivo o ruta de personaje
    let hero_filename = form
        .hero_image_path
        .as_deref()
        .map(|p| std::path::Path::new(p).file_name().and_then(|n| n.to_str()).unwrap_or(p))
        .unwrap_or("Sin arte lateral recortado");

    // Cabecera
    let header_row = row![
        text(title_label)
            .size(20)
            .font(iced::Font::with_name("Segoe UI Bold"))
            .color(Color::WHITE),
        Space::with_width(Length::Fill),
        button(
            svg(svg::Handle::from_memory(include_bytes!("../../../../assets/close-circle-svgrepo-com.svg").as_slice()))
                .width(18)
                .height(18)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
                })
        )
        .padding(4)
        .on_press(Message::CloseCollectionEditor)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    // Campo Nombre
    let name_col = column![
        text("Nombre de la Colección *").size(12).color(Color::from_rgb(0.75, 0.75, 0.8)),
        text_input("Ej. Tomb Raider, Batman, Spider-Man", &form.name)
            .on_input(Message::CollectionEditorNameChanged)
            .padding(10)
            .size(14),
    ]
    .spacing(4);

    // Campo Descripción / Sinopsis
    let desc_col = column![
        text("Sinopsis / Descripción (Hero Showcase)").size(12).color(Color::from_rgb(0.75, 0.75, 0.8)),
        text_input("Breve descripción cinemática que aparecerá en el póster de bienvenida...", &form.description)
            .on_input(Message::CollectionEditorDescriptionChanged)
            .padding(10)
            .size(13),
    ]
    .spacing(4);

    // Campo Fondo Panorámico
    let mut bg_row = row![
        container(
            text(bg_filename)
                .size(12)
                .color(if form.background_image_path.is_some() {
                    Color::from_rgb(0.3, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.6)
                })
        )
        .width(Length::Fill)
        .padding(8)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.13, 0.18))),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        Space::with_width(8),
        button(text("Buscar...").size(12))
            .padding([8, 12])
            .on_press(Message::SelectCollectionBgImage)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.22, 0.24, 0.32))),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .align_y(Alignment::Center);

    if form.background_image_path.is_some() {
        bg_row = bg_row.push(Space::with_width(4)).push(
            button(text("✕").size(12))
                .padding([8, 10])
                .on_press(Message::RemoveCollectionBgImage)
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.5, 0.15, 0.15))),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );
    }

    let bg_section = column![
        text("Imagen de Fondo Panorámica (16:9)").size(12).color(Color::from_rgb(0.75, 0.75, 0.8)),
        bg_row,
    ]
    .spacing(4);

    // Campo Arte Lateral / Personaje
    let mut hero_row = row![
        container(
            text(hero_filename)
                .size(12)
                .color(if form.hero_image_path.is_some() {
                    Color::from_rgb(0.3, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.6)
                })
        )
        .width(Length::Fill)
        .padding(8)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.13, 0.18))),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        Space::with_width(8),
        button(text("Buscar...").size(12))
            .padding([8, 12])
            .on_press(Message::SelectCollectionHeroImage)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.22, 0.24, 0.32))),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .align_y(Alignment::Center);

    if form.hero_image_path.is_some() {
        hero_row = hero_row.push(Space::with_width(4)).push(
            button(text("✕").size(12))
                .padding([8, 10])
                .on_press(Message::RemoveCollectionHeroImage)
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.5, 0.15, 0.15))),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );
    }

    let hero_section = column![
        text("Arte Lateral / Personaje Recortado (PNG Transparente)").size(12).color(Color::from_rgb(0.75, 0.75, 0.8)),
        hero_row,
    ]
    .spacing(4);

    // Campo Icono / Portada
    let mut icon_actions = column![
        button(text("Seleccionar Icono").size(12))
            .padding([6, 12])
            .on_press(Message::SelectCollectionIcon)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.35))),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .spacing(8);

    if form.icon_data.is_some() {
        icon_actions = icon_actions.push(
            button(text("Quitar Icono").size(11))
                .padding([4, 8])
                .on_press(Message::RemoveCollectionIcon)
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    text_color: Color::from_rgb(0.8, 0.3, 0.3),
                    ..Default::default()
                }),
        );
    }

    let icon_section = column![
        text("Icono / Portada Pequeña (1:1)").size(12).color(Color::from_rgb(0.75, 0.75, 0.8)),
        row![
            icon_display,
            icon_actions,
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    // Botones de acción
    let actions_row = row![
        Space::with_width(Length::Fill),
        button(text("Cancelar").size(13))
            .padding([10, 20])
            .on_press(Message::CloseCollectionEditor)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.28))),
                text_color: Color::from_rgb(0.85, 0.85, 0.9),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        button(text("Guardar Colección").size(13).font(iced::Font::with_name("Segoe UI Bold")))
            .padding([10, 24])
            .on_press(Message::SaveCollectionEditor)
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.89, 0.08, 0.12))),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.89, 0.08, 0.12, 0.4),
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }),
    ]
    .spacing(12);

    let modal_content = column![
        header_row,
        name_col,
        desc_col,
        bg_section,
        hero_section,
        icon_section,
        actions_row,
    ]
    .spacing(16)
    .padding(24);

    let scrollable_modal = scrollable(modal_content).height(Length::Shrink);

    let dialog_box = container(scrollable_modal)
        .width(Length::Fixed(520.0))
        .max_height(650.0)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.09, 0.10, 0.14))),
            border: Border {
                radius: 14.0.into(),
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
                width: 1.0,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 25.0,
            },
            ..Default::default()
        });

    container(dialog_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
            ..Default::default()
        })
        .into()
}

