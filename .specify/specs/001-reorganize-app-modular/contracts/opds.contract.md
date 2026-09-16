# Contrato del Protocolo OPDS 1.2

**Feature**: `001-reorganize-app-modular`  
**Fecha**: 2026-09-15  
**Estado**: Completado  

Este documento define la especificación del feed de catálogo estándar **OPDS (Open Publication Distribution System)** versión 1.2 basado en XML Atom, compatible con lectores móviles estándar (Panels, Chunky, Kuro Reader).

---

## 1. Endpoints OPDS

Todos los endpoints requieren autenticación mediante token (por parámetro `?token=<token>` o cabecera `Authorization`).

### A. Catálogo Raíz: `GET /opds`
- **MIME Type**: `application/atom+xml;profile=opds-catalog;kind=navigation;charset=utf-8`
- **Estructura XML de Respuesta**:
```xml
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="http://opds-spec.org/2010/catalog">
  <id>urn:comic:root</id>
  <title>Comic App Library</title>
  <updated>2026-09-15T21:00:00Z</updated>
  <author>
    <name>Comic Server</name>
  </author>
  <link rel="self" href="/opds" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  <link rel="start" href="/opds" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>

  <!-- Entradas de Colecciones -->
  <entry>
    <title>DC Comics</title>
    <id>urn:comic:collection:1</id>
    <updated>2026-09-15T21:00:00Z</updated>
    <content type="text">Colección con 25 cómics</content>
    <link rel="subsection" href="/opds/collections/1" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  </entry>
</feed>
```

---

### B. Catálogo de Adquisición de Colección: `GET /opds/collections/:id`
- **MIME Type**: `application/atom+xml;profile=opds-catalog;kind=acquisition;charset=utf-8`
- **Estructura XML de Respuesta**:
```xml
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:opds="http://opds-spec.org/2010/catalog">
  <id>urn:comic:collection:1</id>
  <title>DC Comics</title>
  <updated>2026-09-15T21:00:00Z</updated>
  <link rel="self" href="/opds/collections/1" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  <link rel="up" href="/opds" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>

  <!-- Entradas de Cómics individuales -->
  <entry>
    <title>Batman #01</title>
    <id>urn:comic:book:10</id>
    <updated>2026-09-15T21:00:00Z</updated>
    <summary>Año: 1940 | Saga: Golden Age | 64 páginas</summary>
    <!-- Enlace a la portada -->
    <link rel="http://opds-spec.org/image" href="/api/comics/10/cover" type="image/jpeg"/>
    <link rel="http://opds-spec.org/image/thumbnail" href="/api/comics/10/cover" type="image/jpeg"/>
    <!-- Enlace de adquisición o lectura -->
    <link rel="http://opds-spec.org/acquisition" href="/api/comics/10/download" type="application/vnd.comicbook+zip"/>
  </entry>
</feed>
```
