//! Generador de feeds XML Atom conformes al estándar OPDS 1.2.

use chrono::Utc;
use crate::domain::collection::Collection;
use crate::domain::comic::Comic;

/// Construye el catálogo raíz OPDS 1.2 (feed de navegación).
pub fn build_root_catalog(collections: &[Collection], token: Option<&str>) -> String {
    let now = Utc::now().to_rfc3339();
    let token_suffix = token.map(|t| format!("?token={}", t)).unwrap_or_default();

    let mut xml = String::with_capacity(2048);
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\" xmlns:opds=\"http://opds-spec.org/2010/catalog\">\n");
    xml.push_str("  <id>urn:comic:root</id>\n");
    xml.push_str("  <title>Comic App Library</title>\n");
    xml.push_str(&format!("  <updated>{}</updated>\n", now));
    xml.push_str("  <author>\n    <name>Comic Server</name>\n  </author>\n");
    xml.push_str(&format!(
        "  <link rel=\"self\" href=\"/opds{}\" type=\"application/atom+xml;profile=opds-catalog;kind=navigation\"/>\n",
        token_suffix
    ));
    xml.push_str(&format!(
        "  <link rel=\"start\" href=\"/opds{}\" type=\"application/atom+xml;profile=opds-catalog;kind=navigation\"/>\n",
        token_suffix
    ));

    for col in collections {
        let safe_name = escape_xml(&col.name);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", safe_name));
        xml.push_str(&format!("    <id>urn:comic:collection:{}</id>\n", col.id));
        xml.push_str(&format!("    <updated>{}</updated>\n", now));
        xml.push_str("    <content type=\"text\">Colección de cómics</content>\n");
        xml.push_str(&format!(
            "    <link rel=\"subsection\" href=\"/opds/collections/{}{}\" type=\"application/atom+xml;profile=opds-catalog;kind=acquisition\"/>\n",
            col.id, token_suffix
        ));
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>");
    xml
}

/// Construye el catálogo de adquisición para una colección específica OPDS 1.2.
pub fn build_collection_catalog(collection: &Collection, comics: &[Comic], token: Option<&str>) -> String {
    let now = Utc::now().to_rfc3339();
    let token_suffix = token.map(|t| format!("?token={}", t)).unwrap_or_default();
    let safe_col_name = escape_xml(&collection.name);

    let mut xml = String::with_capacity(4096);
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\" xmlns:opds=\"http://opds-spec.org/2010/catalog\">\n");
    xml.push_str(&format!("  <id>urn:comic:collection:{}</id>\n", collection.id));
    xml.push_str(&format!("  <title>{}</title>\n", safe_col_name));
    xml.push_str(&format!("  <updated>{}</updated>\n", now));
    xml.push_str(&format!(
        "  <link rel=\"self\" href=\"/opds/collections/{}{}\" type=\"application/atom+xml;profile=opds-catalog;kind=acquisition\"/>\n",
        collection.id, token_suffix
    ));
    xml.push_str(&format!(
        "  <link rel=\"up\" href=\"/opds{}\" type=\"application/atom+xml;profile=opds-catalog;kind=navigation\"/>\n",
        token_suffix
    ));

    for comic in comics {
        let safe_title = escape_xml(&comic.title);
        let year_str = comic.year.map(|y| format!("{}", y)).unwrap_or_else(|| "Desconocido".to_string());
        let saga_str = comic.saga.as_deref().unwrap_or("General");
        let summary = format!("Año: {} | Saga: {} | {} páginas", year_str, saga_str, comic.page_count);
        let safe_summary = escape_xml(&summary);

        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", safe_title));
        xml.push_str(&format!("    <id>urn:comic:book:{}</id>\n", comic.id));
        xml.push_str(&format!("    <updated>{}</updated>\n", now));
        xml.push_str(&format!("    <summary>{}</summary>\n", safe_summary));

        // Enlaces de portada
        xml.push_str(&format!(
            "    <link rel=\"http://opds-spec.org/image\" href=\"/api/comics/{}/cover{}\" type=\"image/jpeg\"/>\n",
            comic.id, token_suffix
        ));
        xml.push_str(&format!(
            "    <link rel=\"http://opds-spec.org/image/thumbnail\" href=\"/api/comics/{}/cover{}\" type=\"image/jpeg\"/>\n",
            comic.id, token_suffix
        ));

        // Enlace de adquisición / lectura (primera página o descarga)
        let mime = match comic.format {
            crate::domain::comic::ComicFormat::Cbz => "application/vnd.comicbook+zip",
            crate::domain::comic::ComicFormat::Cbr => "application/vnd.comicbook-rar",
            crate::domain::comic::ComicFormat::Pdf => "application/pdf",
            crate::domain::comic::ComicFormat::Epub => "application/epub+zip",
        };
        xml.push_str(&format!(
            "    <link rel=\"http://opds-spec.org/acquisition\" href=\"/api/comics/{}/page/0{}\" type=\"{}\"/>\n",
            comic.id, token_suffix, mime
        ));

        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>");
    xml
}

/// Escapa caracteres especiales para contenido XML.
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_root_catalog_contains_expected_tags() {
        let collections = vec![
            Collection {
                id: 1,
                name: "Marvel <Heroes>".to_string(),
                protagonist: None,
                description: None,
                background_image_path: None,
                hero_image_path: None,
                icon_data: None,
                created_at: Utc::now(),
            },
        ];

        let xml = build_root_catalog(&collections, Some("test_tok"));
        assert!(xml.contains("urn:comic:root"));
        assert!(xml.contains("Marvel &lt;Heroes&gt;"));
        assert!(xml.contains("/opds/collections/1?token=test_tok"));
    }

    #[test]
    fn test_build_collection_catalog_contains_expected_tags() {
        let col = Collection {
            id: 1,
            name: "Batman Series".to_string(),
            protagonist: None,
            description: None,
            background_image_path: None,
            hero_image_path: None,
            icon_data: None,
            created_at: Utc::now(),
        };
        let comics = vec![
            Comic {
                id: 42,
                collection_id: 1,
                title: "Batman: Year One".to_string(),
                file_path: std::path::PathBuf::from("batman_01.cbz"),
                format: crate::domain::comic::ComicFormat::Cbz,
                year: Some(1987),
                issue_number: Some(1),
                saga: Some("Year One".to_string()),
                cover_data: None,
                page_count: 32,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let xml = build_collection_catalog(&col, &comics, Some("tok123"));
        assert!(xml.contains("urn:comic:collection:1"));
        assert!(xml.contains("Batman: Year One"));
        assert!(xml.contains("Año: 1987 | Saga: Year One | 32 páginas"));
        assert!(xml.contains("rel=\"http://opds-spec.org/image\""));
        assert!(xml.contains("rel=\"http://opds-spec.org/acquisition\""));
        assert!(xml.contains("/api/comics/42/cover?token=tok123"));
        assert!(xml.contains("/api/comics/42/page/0?token=tok123"));
    }
}
