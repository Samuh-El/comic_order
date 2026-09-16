# Proyecto "Comic"

**Comic** es una aplicación de alto rendimiento para escritorio y servidor local desarrollada en **Rust (Edición 2021)**, diseñada para la catalogación, organización, edición de metadatos, ingesta reactiva y lectura de cómics digitales en formatos empaquetados (`.cbz`, `.cbr`, `.zip`, `.rar`).

---

## Arquitectura del Sistema

El proyecto está diseñado bajo los principios de **Arquitectura Hexagonal (Puertos y Adaptadores / Clean Architecture)** estructurado como un **Monolito Modular**:

```
app/src/
├── domain/                  # Núcleo puro del dominio (entidades, reglas y contratos de puertos)
│   ├── errors.rs            # Tipos de error del dominio
│   ├── comic.rs             # Entidad Comic, formatos y metadatos
│   ├── collection.rs        # Entidad Collection y CollectionPath
│   ├── progress.rs          # Entidad ReadingProgress y porcentaje
│   ├── device.rs            # Entidad TrustedDevice y gestión de tokens
│   └── ports.rs             # Traits de puertos (ComicFileReader, Repositorios, Cache)
│
├── application/             # Servicios de aplicación y orquestación
│   ├── catalog_service.rs   # Casos de uso de catalogación y rutas
│   ├── reader_service.rs    # Casos de uso de lectura, streaming y progreso
│   └── facade.rs            # ComicFacade (Patrón Fachada unificada)
│
├── adapters/
│   ├── inbound/             # Adaptadores de entrada (primarios)
│   │   ├── desktop_ui/      # GUI nativa en Iced (TEA) con zoom y paneo continuo
│   │   ├── rest_api/        # API REST en Axum con streaming responsivo y SPA sin CDNs
│   │   └── opds/            # Feed estándar OPDS 1.2 en XML Atom
│   │
│   └── outbound/            # Adaptadores de salida (secundarios)
│       ├── persistence/     # SQLite local asíncrono con SQLx (modo WAL)
│       ├── file_readers/    # Adaptadores desacoplados para CBZ (zip) y CBR (unrar)
│       └── cache/           # Proxy de caché de imágenes en disco (`.cache/`)
│
├── ingestion/               # Módulo de ingesta reactiva y File Watcher
│   ├── comic_info.rs        # Extractor y parseador de ComicInfo.xml
│   ├── watcher.rs           # File Watcher con notify y 500 ms de debouncing
│   └── worker.rs            # IngestionWorker asíncrono en segundo plano
│
├── network/                 # Utilidades de red local (LAN)
│   ├── qr.rs                # Generación de códigos QR matriciales y detección de IP
│   └── mdns.rs              # Anuncio de servicio local mDNS (_comic._tcp.local)
│
└── main.rs                  # Punto de entrada, composición e inyección de dependencias
```

---

## Características Principales

- **Lectura en Escritorio de Alto Rendimiento**: Visor gráfico acelerado por hardware con navegación secuencial rápida, atajos de teclado, zoom dinámico continuo (10% a 500%) y paneo por arrastre.
- **Acceso Remoto Multidispositivo (Servidor Web + QR)**: Servidor Axum embebido en el puerto `8080`, sincronización instantánea por código QR y autenticación mediante tokens de sesión o dispositivos de confianza (*trusted devices*).
- **Lector Web Autónomo**: SPA ligera servida localmente en HTML5, CSS3 y JavaScript moderno, 100% autónoma sin CDNs ni librerías externas.
- **Protocolo Estándar OPDS 1.2**: Feed Atom XML compatible con clientes móviles de terceros (Panels, Chunky, Kuro Reader) en `/opds`.
- **Descubrimiento mDNS / Zeroconf**: Anuncio automático en red local bajo el identificador `_comic._tcp.local`.
- **Ingesta Reactiva Asíncrona**: Monitoreo de carpetas en tiempo real con `notify` (500 ms debounce), extracción automática de `ComicInfo.xml` y generación de portadas sin bloquear la interfaz.
- **Persistencia Robusta**: Base de datos SQLite local (`app/data/comic.db`) con modo WAL (`PRAGMA journal_mode=WAL;`), claves foráneas y consultas 100% parametrizadas.
- **Privacidad y Autonomía (Offline-First)**: Sin telemetría, sin dependencias en la nube y estrictamente local.

---

## Requisitos del Sistema

- **Rust**: Versión estable (2021 edition) — [https://rustup.rs](https://rustup.rs)
- **Windows**: Visual Studio Build Tools 2022 con el componente "Desarrollo para el escritorio con C++" y Windows 11 SDK.

---

## Compilación y Pruebas

Todos los comandos se ejecutan desde el directorio `app/`:

```powershell
cd C:\Workspace\comic_order\app

# Compilar y verificar tipos
cargo check

# Ejecutar suite completa de pruebas unitarias
cargo test

# Iniciar la aplicación
cargo run
```

O utilizando los scripts automatizados:
- `.\app\scripts\build.cmd`: Configura el entorno MSVC y compila.
- `.\app\scripts\genera_exe.cmd`: Genera el ejecutable autónomo para distribución.

---

## Protocolo OPDS y Lector Remoto

1. **Acceso Web Remoto**: Escanear el código QR desplegado en la app o acceder a `http://<IP_LOCAL>:8080/?token=<TOKEN>`.
2. **Catálogo OPDS**: Ingresar en lectores compatibles la dirección `http://<IP_LOCAL>:8080/opds?token=<TOKEN>`.
