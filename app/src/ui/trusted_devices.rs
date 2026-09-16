use iced::widget::{button, column, container, row, scrollable, text, Space, svg};
use iced::{Alignment, Element, Length, Theme};

use crate::db::TrustedDevice;
use crate::Message;

pub struct TrustedDevicesForm {
    pub devices: Vec<TrustedDevice>,
}

impl TrustedDevicesForm {
    pub fn new(devices: Vec<TrustedDevice>) -> Self {
        Self { devices }
    }
}

pub fn view<'a>(form: &'a TrustedDevicesForm, qr_handle: Option<&'a iced::widget::image::Handle>, qr_url: Option<&'a str>) -> Element<'a, Message> {
    let mut list = column![].spacing(10);

    if form.devices.is_empty() && qr_handle.is_none() {
        list = list.push(
            container(text("No hay dispositivos recurrentes guardados.").size(14))
                .width(Length::Fill)
                .padding(20)
                .center_x(Length::Fill),
        );
    } else {
        for device in &form.devices {
            let item = container(
                row![
                    column![
                        text(&device.device_name).size(16).font(iced::Font::with_name("Segoe UI Semibold")),
                        text(format!("Token: {}...", &device.token[..8])).size(12).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        text(format!("Desde: {}", device.created_at)).size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    button(
                        svg(svg::Handle::from_memory(include_bytes!("../../assets/delete-1487-svgrepo-com.svg").as_slice()))
                            .width(16)
                            .height(16)
                            .style(|_theme: &Theme, _status| svg::Style {
                                color: Some(iced::Color::from_rgb(0.91, 0.27, 0.37)),
                            })
                    )
                    .padding(8)
                    .on_press(Message::DeleteTrustedDevice(device.id))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .align_y(Alignment::Center)
                .padding(12)
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.16))),
                border: iced::Border {
                    radius: 10.0.into(),
                    color: iced::Color::from_rgb(0.2, 0.2, 0.25),
                    width: 1.0,
                },
                ..Default::default()
            });

            list = list.push(item);
        }
    }

    let header = row![
        text("Dispositivos Recurrentes").size(24).font(iced::Font::with_name("Segoe UI Bold")),
        Space::with_width(Length::Fill),
        button(
            row![
                text("+").size(18),
                text("Añadir").size(13)
            ]
            .spacing(5)
            .align_y(Alignment::Center)
        )
        .padding([6, 12])
        .on_press(Message::AddTrustedDevice)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.91, 0.27, 0.37))),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        Space::with_width(Length::Fixed(10.0)),
        button(
            svg(svg::Handle::from_memory(include_bytes!("../../assets/close-circle-svgrepo-com.svg").as_slice()))
                .width(20)
                .height(20)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                })
        )
        .on_press(Message::CloseTrustedDevices)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    let mut body = column![
        header,
        text("Estos dispositivos pueden conectarse sin escanear QR mientras la app esté abierta.").size(13).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
        scrollable(list).height(Length::Fixed( jika_qr(300.0, 200.0, qr_handle.is_some()) )),
    ]
    .spacing(20);

    if let Some(handle) = qr_handle {
        let qr_box = column![
            text("¡Nuevo Dispositivo!").size(18).color(iced::Color::from_rgb(0.91, 0.27, 0.37)),
            text("Escanea este QR con el nuevo dispositivo:").size(12).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            container(iced::widget::image(handle.clone()).width(180).height(180))
                .padding(10)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::WHITE)),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text(qr_url.unwrap_or("")).size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        body = body.push(qr_box);
    }

    container(
        container(body)
            .width(450)
            .padding(25)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.08, 0.08, 0.1))),
                border: iced::Border {
                    radius: 16.0.into(),
                    color: iced::Color::from_rgb(0.91, 0.27, 0.37),
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
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.8))),
        ..Default::default()
    })
    .into()
}

fn jika_qr(normal: f32, with_qr: f32, is_qr: bool) -> f32 {
    if is_qr { with_qr } else { normal }
}
