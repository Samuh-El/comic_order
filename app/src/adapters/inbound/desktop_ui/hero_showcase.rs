//! Componente Home Hero Showcase para la pantalla de inicio.
//!
//! Implementa una experiencia cinematográfica multicapa (stack!) inspirada en pósteres
//! de alta fidelidad, con rotación automática, Hover Pause, animación de transición,
//! carrusel inferior de últimas colecciones y estado vacío elegante.

use std::collections::HashMap;
use std::path::Path;
use iced::widget::{button, column, container, image, mouse_area, row, stack, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme, Vector};

use crate::domain::collection::Collection;

/// Estado visual del Showcase Hero para The Elm Architecture (TEA).
#[derive(Debug, Clone)]
pub struct HeroShowcaseState {
    pub recent_collections: Vec<Collection>,
    pub active_index: usize,
    pub elapsed_secs: f32,
    pub is_hovered: bool,
    pub transition_progress: Option<f32>,
    pub previous_index: Option<usize>,
    pub bg_handles: HashMap<i64, image::Handle>,
    pub hero_handles: HashMap<i64, image::Handle>,
}

impl Default for HeroShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

impl HeroShowcaseState {
    pub fn new() -> Self {
        Self {
            recent_collections: Vec::new(),
            active_index: 0,
            elapsed_secs: 0.0,
            is_hovered: false,
            transition_progress: None,
            previous_index: None,
            bg_handles: HashMap::new(),
            hero_handles: HashMap::new(),
        }
    }

    /// Actualiza la lista de colecciones recientes manteniendo la selección actual si es válida.
    pub fn set_collections(&mut self, collections: Vec<Collection>) {
        self.recent_collections = collections;
        if self.active_index >= self.recent_collections.len() {
            self.active_index = 0;
        }
        self.load_missing_handles();
    }

    /// Carga las imágenes de fondo y lateral para las colecciones actuales si no están cacheadas.
    pub fn load_missing_handles(&mut self) {
        for col in &self.recent_collections {
            if !self.bg_handles.contains_key(&col.id) {
                if let Some(bg_path) = &col.background_image_path {
                    if Path::new(bg_path).exists() {
                        self.bg_handles.insert(col.id, image::Handle::from_path(bg_path));
                    }
                }
            }
            if !self.hero_handles.contains_key(&col.id) {
                if let Some(hero_path) = &col.hero_image_path {
                    if Path::new(hero_path).exists() {
                        self.hero_handles.insert(col.id, image::Handle::from_path(hero_path));
                    }
                }
            }
        }
    }

    /// Retorna la colección activa actual, si existe.
    pub fn active_collection(&self) -> Option<&Collection> {
        self.recent_collections.get(self.active_index)
    }

    /// Selecciona una colección por su índice e inicia la animación.
    pub fn select_index(&mut self, target_idx: usize) {
        if target_idx < self.recent_collections.len() && target_idx != self.active_index {
            self.previous_index = Some(self.active_index);
            self.active_index = target_idx;
            self.elapsed_secs = 0.0;
            self.transition_progress = Some(0.0);
        }
    }

    /// Avanza a la siguiente colección cíclicamente.
    pub fn advance_next(&mut self) {
        if self.recent_collections.len() >= 2 {
            let next_idx = (self.active_index + 1) % self.recent_collections.len();
            self.select_index(next_idx);
        }
    }
}

/// Mensajes de interacción emitidos por el Hero Showcase.
#[derive(Debug, Clone)]
pub enum HeroShowcaseMessage {
    AutoplayTick,
    HoverChanged(bool),
    SelectCollection(usize),
    TransitionTick(f32),
    ReadNowClicked(i64),
    CreateFirstCollectionClicked,
}

/// Renderiza la vista del Hero Showcase o el estado vacío.
pub fn view<'a, Message>(
    state: &'a HeroShowcaseState,
    map_msg: impl Fn(HeroShowcaseMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    if state.recent_collections.is_empty() {
        return view_empty_state(map_msg);
    }

    let Some(active_col) = state.active_collection() else {
        return view_empty_state(map_msg);
    };

    let anim_factor = state.transition_progress.unwrap_or(1.0);
    let slide_offset = (1.0 - anim_factor) * 40.0;

    // 1. Capa de Fondo (Background Layer)
    let bg_element: Element<'a, Message> = if let Some(handle) = state.bg_handles.get(&active_col.id) {
        container(
            image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.05, 0.06, 0.08))),
                ..Default::default()
            })
            .into()
    };

    // 2. Viñeteado cinemático oscuro y degradado sobre el fondo
    let vignette_layer: Element<'a, Message> = container(Space::new(Length::Fill, Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.04, 0.04, 0.06, 0.55))),
            ..Default::default()
        })
        .into();

    // 3. Personaje lateral derecho (Hero Art)
    let hero_art_element: Element<'a, Message> = if let Some(handle) = state.hero_handles.get(&active_col.id) {
        container(
            image(handle.clone())
                .height(Length::Fixed(600.0))
                .content_fit(iced::ContentFit::Contain),
        )
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 40.0,
            right: 60.0,
            bottom: 110.0,
            left: 0.0,
        })
        .into()
    } else if let Some(icon_data) = &active_col.icon_data {
        let handle = image::Handle::from_bytes(icon_data.clone());
        container(
            container(
                image(handle)
                    .width(Length::Fixed(280.0))
                    .height(Length::Fixed(420.0))
                    .content_fit(iced::ContentFit::Cover),
            )
            .style(|_theme: &Theme| container::Style {
                border: Border {
                    color: Color::from_rgba(0.9, 0.1, 0.15, 0.8),
                    width: 2.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
                    offset: Vector::new(8.0, 12.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }),
        )
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 20.0,
            right: 80.0,
            bottom: 100.0,
            left: 0.0,
        })
        .into()
    } else {
        // Fallback temático: Tarjeta estilizada con relieve
        container(
            container(
                column![
                    text("COLECCIÓN").size(14).color(Color::from_rgb(0.9, 0.2, 0.2)),
                    text(&active_col.name)
                        .size(24)
                        .color(Color::WHITE)
                        .font(iced::Font::with_name("Segoe UI Semibold")),
                    Space::with_height(10),
                    text("EDICIÓN CINEMÁTICA").size(11).color(Color::from_rgb(0.6, 0.6, 0.7)),
                ]
                .spacing(6)
                .align_x(Alignment::Center),
            )
            .width(Length::Fixed(260.0))
            .height(Length::Fixed(380.0))
            .center_x(Length::Fixed(260.0))
            .center_y(Length::Fixed(380.0))
            .padding(20)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.08, 0.09, 0.13, 0.85))),
                border: Border {
                    color: Color::from_rgba(0.9, 0.1, 0.15, 0.6),
                    width: 1.5,
                    radius: 10.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: Vector::new(4.0, 8.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            }),
        )
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 0.0,
            right: 80.0,
            bottom: 100.0,
            left: 0.0,
        })
        .into()
    };

    // 4. Panel Frontal Izquierdo (Badge, Título en Relieve, Sinopsis y Botón)
    let badge = create_credential_badge();

    // Título estilizado con efecto de relieve y sombra roja 3D
    let title_text = active_col.name.to_uppercase();
    let title_relief = stack![
        // Sombra roja desplazada (+3, +3)
        container(
            text(title_text.clone())
                .size(46)
                .font(iced::Font::with_name("Segoe UI Black"))
                .color(Color::from_rgb(0.9, 0.08, 0.12)),
        )
        .padding(iced::Padding {
            top: 3.0,
            left: 3.0,
            right: 0.0,
            bottom: 0.0,
        }),
        // Texto frontal blanco brillante
        text(title_text)
            .size(46)
            .font(iced::Font::with_name("Segoe UI Black"))
            .color(Color::WHITE),
    ];

    // Panel de Sinopsis con efecto Glassmorphism
    let description_content = if let Some(desc) = &active_col.description {
        if !desc.trim().is_empty() {
            desc.as_str()
        } else {
            "Explora los tomos, arcos argumentales y ediciones especiales de esta colección. Sumérgete en una experiencia de lectura cinematográfica con renderizado de alta fidelidad."
        }
    } else {
        "Explora los tomos, arcos argumentales y ediciones especiales de esta colección. Sumérgete en una experiencia de lectura cinematográfica con renderizado de alta fidelidad."
    };

    let synopsis_panel = container(
        column![
            text("SINOPSIS")
                .size(11)
                .font(iced::Font::with_name("Segoe UI Bold"))
                .color(Color::from_rgb(0.9, 0.25, 0.25)),
            Space::with_height(6),
            text(description_content)
                .size(14)
                .color(Color::from_rgb(0.86, 0.88, 0.92))
                .line_height(iced::widget::text::LineHeight::Relative(1.4)),
        ]
        .spacing(2),
    )
    .width(Length::Fixed(500.0))
    .padding(20)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.07, 0.10, 0.72))),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    });

    // Botón poligonal estilizado "READ NOW" con corte y flecha
    let col_id = active_col.id;
    let read_now_btn = button(
        row![
            text("READ NOW")
                .size(15)
                .font(iced::Font::with_name("Segoe UI Bold"))
                .color(Color::WHITE),
            Space::with_width(12),
            text("▶").size(13).color(Color::WHITE),
        ]
        .align_y(Alignment::Center),
    )
    .padding(iced::Padding {
        top: 14.0,
        bottom: 14.0,
        left: 28.0,
        right: 28.0,
    })
    .on_press(map_msg(HeroShowcaseMessage::ReadNowClicked(col_id)))
    .style(|_theme: &Theme, status| {
        let (bg_color, shadow_blur) = match status {
            button::Status::Hovered => (Color::from_rgb(1.0, 0.15, 0.22), 16.0),
            button::Status::Pressed => (Color::from_rgb(0.75, 0.06, 0.1), 8.0),
            _ => (Color::from_rgb(0.89, 0.04, 0.08), 12.0),
        };

        button::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.89, 0.04, 0.08, 0.45),
                offset: Vector::new(0.0, 4.0),
                blur_radius: shadow_blur,
            },
            text_color: Color::WHITE,
        }
    });

    let hero_left_content: Element<'a, Message> = column![
        badge,
        Space::with_height(16),
        title_relief,
        Space::with_height(16),
        synopsis_panel,
        Space::with_height(24),
        read_now_btn,
    ]
    .spacing(0)
    .padding(iced::Padding {
        top: 40.0,
        left: 60.0 + slide_offset,
        bottom: 120.0,
        right: 0.0,
    })
    .into();

    // 5. Franja Horizontal Inferior (Carrusel de 5 Colecciones Recientes)
    let carousel = view_bottom_carousel(state, map_msg);
    let carousel_layer: Element<'a, Message> = container(carousel)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(iced::Padding {
            top: 0.0,
            left: 60.0,
            bottom: 20.0,
            right: 60.0,
        })
        .into();

    let main_composition = stack![
        bg_element,
        vignette_layer,
        hero_art_element,
        hero_left_content,
        carousel_layer,
    ];

    // Envolver con mouse_area para la funcionalidad de Hover Pause
    mouse_area(main_composition)
        .on_enter(map_msg(HeroShowcaseMessage::HoverChanged(true)))
        .on_exit(map_msg(HeroShowcaseMessage::HoverChanged(false)))
        .into()
}

/// Renderiza la franja horizontal inferior con tarjetas de las 5 colecciones más recientes.
fn view_bottom_carousel<'a, Message>(
    state: &'a HeroShowcaseState,
    map_msg: impl Fn(HeroShowcaseMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let mut cards = row![].spacing(14).align_y(Alignment::Center);

    for (idx, col) in state.recent_collections.iter().enumerate().take(5) {
        let is_active = idx == state.active_index;

        let icon_preview: Element<'a, Message> = if let Some(icon) = &col.icon_data {
            let handle = image::Handle::from_bytes(icon.clone());
            image(handle)
                .width(Length::Fixed(46.0))
                .height(Length::Fixed(56.0))
                .content_fit(iced::ContentFit::Cover)
                .into()
        } else {
            container(
                text("C")
                    .size(18)
                    .font(iced::Font::with_name("Segoe UI Bold"))
                    .color(Color::from_rgb(0.9, 0.2, 0.2)),
            )
            .width(Length::Fixed(46.0))
            .height(Length::Fixed(56.0))
            .center_x(Length::Fixed(46.0))
            .center_y(Length::Fixed(56.0))
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.12, 0.14, 0.20, 0.9))),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        };

        let title_label = text(&col.name)
            .size(13)
            .font(iced::Font::with_name(if is_active {
                "Segoe UI Bold"
            } else {
                "Segoe UI"
            }))
            .color(if is_active {
                Color::WHITE
            } else {
                Color::from_rgb(0.75, 0.77, 0.82)
            });

        let card_content = row![
            icon_preview,
            Space::with_width(10),
            column![
                title_label,
                text(if is_active { "EN REPRODUCCIÓN" } else { "COLECCIÓN" })
                    .size(9)
                    .color(if is_active {
                        Color::from_rgb(0.9, 0.2, 0.25)
                    } else {
                        Color::from_rgb(0.5, 0.52, 0.58)
                    }),
            ]
            .spacing(3)
            .align_x(Alignment::Start),
        ]
        .align_y(Alignment::Center);

        let card_btn = button(card_content)
            .padding(iced::Padding {
                top: 6.0,
                bottom: 6.0,
                left: 10.0,
                right: 14.0,
            })
            .on_press(map_msg(HeroShowcaseMessage::SelectCollection(idx)))
            .style(move |_theme: &Theme, status| {
                let is_hovered = matches!(status, button::Status::Hovered);

                let bg_color = if is_active {
                    Color::from_rgba(0.14, 0.16, 0.22, 0.88)
                } else if is_hovered {
                    Color::from_rgba(0.10, 0.12, 0.16, 0.75)
                } else {
                    Color::from_rgba(0.06, 0.07, 0.10, 0.65)
                };

                let border_color = if is_active {
                    Color::from_rgb(0.9, 0.1, 0.15)
                } else if is_hovered {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.3)
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.10)
                };

                let shadow = if is_active {
                    Shadow {
                        color: Color::from_rgba(0.9, 0.1, 0.15, 0.4),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 10.0,
                    }
                } else {
                    Shadow::default()
                };

                button::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: if is_active { 1.8 } else { 1.0 },
                        radius: 8.0.into(),
                    },
                    shadow,
                    text_color: Color::WHITE,
                }
            });

        cards = cards.push(card_btn);
    }

    container(cards)
        .padding(iced::Padding {
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.04, 0.05, 0.07, 0.80))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}

/// Crea el badge gráfico tipo credencial ("HELLO my name is...").
fn create_credential_badge<'a, Message>() -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let red_header = container(
        column![
            text("HELLO")
                .size(11)
                .font(iced::Font::with_name("Segoe UI Black"))
                .color(Color::WHITE),
            text("MY NAME IS")
                .size(8)
                .font(iced::Font::with_name("Segoe UI Bold"))
                .color(Color::from_rgb(1.0, 0.9, 0.9)),
        ]
        .spacing(0)
        .align_x(Alignment::Center),
    )
    .width(Length::Fixed(150.0))
    .padding(iced::Padding {
        top: 4.0,
        bottom: 4.0,
        left: 8.0,
        right: 8.0,
    })
    .center_x(Length::Fixed(150.0))
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.89, 0.06, 0.12))),
        border: Border {
            radius: iced::border::Radius {
                top_left: 6.0,
                top_right: 6.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            ..Default::default()
        },
        ..Default::default()
    });

    let white_body = container(
        text("FEATURED")
            .size(11)
            .font(iced::Font::with_name("Segoe UI Black"))
            .color(Color::from_rgb(0.1, 0.1, 0.14)),
    )
    .width(Length::Fixed(150.0))
    .padding(iced::Padding {
        top: 6.0,
        bottom: 6.0,
        left: 8.0,
        right: 8.0,
    })
    .center_x(Length::Fixed(150.0))
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.96, 0.96, 0.98))),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 6.0,
                bottom_right: 6.0,
            },
            ..Default::default()
        },
        ..Default::default()
    });

    container(column![red_header, white_body])
        .style(|_theme: &Theme| container::Style {
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        })
        .into()
}

/// Renderiza la vista de estado vacío cuando no existen colecciones registradas.
pub fn view_empty_state<'a, Message>(
    map_msg: impl Fn(HeroShowcaseMessage) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let title = text("Crea tu primera colección")
        .size(36)
        .font(iced::Font::with_name("Segoe UI Bold"))
        .color(Color::WHITE);

    let subtitle = text(
        "Organiza tus cómics favoritos en colecciones personalizadas con arte cinematográfico y portadas de alta definición.",
    )
    .size(15)
    .color(Color::from_rgb(0.65, 0.67, 0.73));

    let create_btn = button(
        row![
            text("+").size(18).color(Color::WHITE),
            Space::with_width(8),
            text("NUEVA COLECCIÓN")
                .size(14)
                .font(iced::Font::with_name("Segoe UI Bold"))
                .color(Color::WHITE),
        ]
        .align_y(Alignment::Center),
    )
    .padding(iced::Padding {
        top: 14.0,
        bottom: 14.0,
        left: 28.0,
        right: 28.0,
    })
    .on_press(map_msg(HeroShowcaseMessage::CreateFirstCollectionClicked))
    .style(|_theme: &Theme, status| {
        let (bg, shadow_blur) = match status {
            button::Status::Hovered => (Color::from_rgb(1.0, 0.15, 0.22), 16.0),
            button::Status::Pressed => (Color::from_rgb(0.75, 0.05, 0.10), 8.0),
            _ => (Color::from_rgb(0.89, 0.04, 0.08), 12.0),
        };

        button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba(0.89, 0.04, 0.08, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: shadow_blur,
            },
            text_color: Color::WHITE,
        }
    });

    container(
        column![
            title,
            Space::with_height(12),
            subtitle,
            Space::with_height(32),
            create_btn,
        ]
        .spacing(0)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.04, 0.04, 0.06))),
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_state_empty_initialization() {
        let state = HeroShowcaseState::new();
        assert!(state.recent_collections.is_empty());
        assert_eq!(state.active_index, 0);
        assert!(!state.is_hovered);
        assert!(state.transition_progress.is_none());
        assert!(state.active_collection().is_none());
    }

    #[test]
    fn test_hero_state_select_and_advance() {
        let mut state = HeroShowcaseState::new();
        let cols = vec![
            Collection::new(1, "Coleccion 1".to_string()).expect("valida"),
            Collection::new(2, "Coleccion 2".to_string()).expect("valida"),
            Collection::new(3, "Coleccion 3".to_string()).expect("valida"),
        ];
        state.set_collections(cols);

        assert_eq!(state.recent_collections.len(), 3);
        assert_eq!(state.active_index, 0);
        assert_eq!(state.active_collection().expect("existe").name, "Coleccion 1");

        // Seleccionar índice manual
        state.select_index(2);
        assert_eq!(state.active_index, 2);
        assert_eq!(state.previous_index, Some(0));
        assert_eq!(state.transition_progress, Some(0.0));
        assert_eq!(state.active_collection().expect("existe").name, "Coleccion 3");

        // Avanzar cíclicamente
        state.advance_next();
        assert_eq!(state.active_index, 0);
        assert_eq!(state.previous_index, Some(2));
    }
}
