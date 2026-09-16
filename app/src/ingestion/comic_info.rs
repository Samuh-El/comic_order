//! Extracción y parseo de metadatos desde archivos `ComicInfo.xml` embebidos en cómics.

use crate::domain::comic::ComicMetadata;

/// Estructura de metadatos extraída desde `ComicInfo.xml`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedComicInfo {
    pub title: Option<String>,
    pub series: Option<String>,
    pub number: Option<i32>,
    pub year: Option<i32>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub summary: Option<String>,
    pub publisher: Option<String>,
    pub page_count: Option<usize>,
}

impl ParsedComicInfo {
    /// Convierte los metadatos parseados en el tipo de dominio `ComicMetadata`.
    pub fn to_domain_metadata(&self) -> ComicMetadata {
        ComicMetadata {
            series: self.series.clone(),
            writer: self.writer.clone(),
            penciller: self.penciller.clone(),
            summary: self.summary.clone(),
            publisher: self.publisher.clone(),
            tags: Vec::new(),
        }
    }

    /// Parsea el contenido XML de un archivo `ComicInfo.xml`.
    pub fn from_xml(xml_content: &str) -> Self {
        let mut info = Self::default();

        info.title = extract_tag_value(xml_content, "Title");
        info.series = extract_tag_value(xml_content, "Series");
        info.number = extract_tag_value(xml_content, "Number")
            .and_then(|v| v.parse::<i32>().ok());
        info.year = extract_tag_value(xml_content, "Year")
            .and_then(|v| v.parse::<i32>().ok());
        info.writer = extract_tag_value(xml_content, "Writer");
        info.penciller = extract_tag_value(xml_content, "Penciller");
        info.summary = extract_tag_value(xml_content, "Summary");
        info.publisher = extract_tag_value(xml_content, "Publisher");
        info.page_count = extract_tag_value(xml_content, "PageCount")
            .and_then(|v| v.parse::<usize>().ok());

        info
    }
}

/// Extrae el texto interno de una etiqueta XML simple (`<Tag>valor</Tag>`), manejando CDATA y entidades.
fn extract_tag_value(xml: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);

    let start_idx = xml.find(&open_tag)? + open_tag.len();
    let end_idx = xml[start_idx..].find(&close_tag)? + start_idx;

    let raw_text = xml[start_idx..end_idx].trim();
    if raw_text.is_empty() {
        return None;
    }

    // Limpieza de bloques <![CDATA[ ... ]]>
    let cleaned = if let Some(stripped) = raw_text.strip_prefix("<![CDATA[") {
        if let Some(end_cdata) = stripped.strip_suffix("]]>") {
            end_cdata.trim()
        } else {
            stripped.trim()
        }
    } else {
        raw_text
    };

    Some(decode_xml_entities(cleaned))
}

/// Decodifica entidades XML estándar.
fn decode_xml_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comic_info_standard() {
        let xml = r#"<?xml version="1.0"?>
        <ComicInfo>
            <Title>El Regreso del Caballero Oscuro</Title>
            <Series>Batman</Series>
            <Number>1</Number>
            <Year>1986</Year>
            <Writer>Frank Miller</Writer>
            <Publisher>DC Comics</Publisher>
            <Summary>Una historia legendaria de Bruce Wayne retirado.</Summary>
            <PageCount>48</PageCount>
        </ComicInfo>"#;

        let parsed = ParsedComicInfo::from_xml(xml);
        assert_eq!(parsed.title.as_deref(), Some("El Regreso del Caballero Oscuro"));
        assert_eq!(parsed.series.as_deref(), Some("Batman"));
        assert_eq!(parsed.number, Some(1));
        assert_eq!(parsed.year, Some(1986));
        assert_eq!(parsed.writer.as_deref(), Some("Frank Miller"));
        assert_eq!(parsed.publisher.as_deref(), Some("DC Comics"));
        assert_eq!(parsed.page_count, Some(48));
    }

    #[test]
    fn test_parse_comic_info_cdata() {
        let xml = r#"<ComicInfo>
            <Title><![CDATA[Spider-Man & Venom]]></Title>
            <Summary><![CDATA[Batalla en Nueva York con <efectos especiales>]]></Summary>
        </ComicInfo>"#;

        let parsed = ParsedComicInfo::from_xml(xml);
        assert_eq!(parsed.title.as_deref(), Some("Spider-Man & Venom"));
        assert_eq!(parsed.summary.as_deref(), Some("Batalla en Nueva York con <efectos especiales>"));
    }
}
