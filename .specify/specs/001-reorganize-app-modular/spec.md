# Especificación de Característica: Reorganización Modular Hexagonal de la Aplicación

**Feature Branch**: `001-reorganize-app-modular`  
**Created**: 2026-09-15  
**Status**: Draft  
**Input**: Reorganizar el proyecto de la carpeta `app` manteniendo todas las funcionalidades actuales pero estructurándolo como un Monolito Modular con Arquitectura Hexagonal (Puertos y Adaptadores), capas desacopladas, patrones de diseño robustos (Adaptador, Fachada, Observador, Estrategia, Proxy/Caché) y capacidades avanzadas de red local (streaming de páginas bajo demanda, mDNS y control de concurrencia).

---

## Clarificaciones

### Sesión 2026-09-15
- **Q1: ¿Cuál debe ser la estrategia de refactorización para reorganizar el código actual de `app/src/`?** → **A:** Migración incremental por capas: se define primero el núcleo de dominio y contratos de puertos, luego los adaptadores de salida (SQLite y lectores CBZ/CBR), la fachada de aplicación y finalmente los adaptadores de entrada (GUI Iced y Axum REST), validando la compilación limpia en cada etapa.
- **Q2: ¿Cómo debe implementar el `ImageCacheProxy` la retención y entrega de páginas y miniaturas procesadas bajo demanda?** → **A:** Caché persistente en disco en `app/.cache/`: las páginas y miniaturas extraídas y redimensionadas bajo demanda se almacenan en un directorio de caché local en disco con mecanismo de limpieza periódica o límite de tamaño máximo para acelerar relecturas subsiguientes sin sobrecargar memoria RAM.
- **Q3: ¿Qué mecanismo debe utilizar el `IngestionWorker` para detectar cómics añadidos, modificados o eliminados en las carpetas de colección?** → **A:** Eventos nativos del sistema operativo (`notify`) con canal asíncrono Tokio `mpsc` y ventana de debouncing de 500 ms para reaccionar al instante con 0% de uso de CPU en reposo, complementado con un escaneo inicial de verificación al iniciar la aplicación.
- **Q4: ¿Cuál debe ser el alcance de entrega de los puertos de entrada OPDS y WebSockets en esta reorganización?** → **A:** Feed OPDS 1.2 funcional con endpoints `/opds` para catálogo Atom XML (navegación de colecciones, portadas y adquisición de volúmenes) utilizable por lectores móviles estándar, y definición formal del puerto/contrato de dominio para WebSockets de progreso de lectura.

---

## Escenarios de Usuario y Pruebas *(Obligatorio)*

### Historia de Usuario 1 — Lectura y Navegación Local de Cómics sin Regresiones (Prioridad: P1)

Como usuario lector de escritorio, deseo abrir mi biblioteca de cómics, navegar por mis colecciones existentes, ver las portadas al instante y leer cualquier cómic (CBZ/CBR) con controles de zoom y desplazamiento fluidos, exactamente como lo hacía en la versión anterior pero con una arquitectura interna limpia que garantice estabilidad y rapidez.

**Por qué esta prioridad**: Es la funcionalidad central y el valor primordial del producto; ninguna reorganización estructural puede degradar ni romper la experiencia de lectura nativa actual.

**Prueba Independiente**: Se puede probar abriendo la aplicación de escritorio en `app/`, seleccionando una colección cargada previamente, abriendo un tomo y navegando entre páginas usando teclado y ratón con respuesta inmediata (< 100 ms).

**Escenarios de Aceptación**:
1. **Dado** que el usuario inicia la aplicación de escritorio y selecciona una colección con cómics indexados, **Cuando** se visualiza la cuadrícula, **Entonces** todas las carátulas se renderizan al instante desde la caché de persistencia sin lecturas innecesarias de disco.
2. **Dado** que el usuario abre un cómic en formato `.cbz` o `.cbr`, **Cuando** navega hacia adelante o atrás con las flechas o botones, **Entonces** la página solicitada se extrae selectivamente en memoria RAM y se muestra sin parpadeos ni bloqueos de la interfaz gráfica.
3. **Dado** que el usuario utiliza la rueda del ratón o arrastra la pantalla, **Cuando** interactúa sobre la página abierta, **Entonces** el visor ajusta el zoom (entre 10% y 500%) y traslada la imagen en tiempo real mediante el motor de renderizado gráfico.

---

### Historia de Usuario 2 — Servidor Web Local y Lectura Remota Multidispositivo (Prioridad: P1)

Como usuario que lee desde su teléfono móvil o tableta en la misma red Wi-Fi, deseo escanear el código QR en la aplicación de escritorio para abrir el lector web responsivo, acceder a todas mis colecciones y leer cualquier volumen mediante streaming de páginas bajo demanda optimizadas para la resolución de mi dispositivo.

**Por qué esta prioridad**: Es la característica diferencial del producto, permitiendo la lectura remota doméstica sin depender de servicios en la nube ni de instalaciones externas.

**Prueba Independiente**: Iniciar el servidor local desde la GUI de escritorio, escanear el QR con un dispositivo móvil en la misma red LAN, y verificar que la SPA web cargue la lista de colecciones y permita leer páginas deslizando el dedo sin errores de autenticación.

**Escenarios de Aceptación**:
1. **Dado** que el usuario presiona "Compartir QR", **Cuando** el servidor HTTP arranca, **Entonces** se expone un token de sesión efímero o persistente y se despliega un código QR con la URL local correcta (`http://<IP_LAN>:8080/?token=<TOKEN>`).
2. **Dado** que un navegador remoto realiza peticiones con el token autorizado, **Cuando** solicita una página específica de un cómic, **Entonces** el adaptador de salida obtiene la página requerida y el proxy de caché la sirve de forma optimizada.
3. **Dado** que se registra un dispositivo de confianza (*trusted device*) con su token permanente, **Cuando** el usuario vuelve a conectarse sin reescanear el QR, **Entonces** el servidor valida su credencial contra la base de datos y le concede acceso continuo.

---

### Historia de Usuario 3 — Ingesta Asíncrona, File Watcher y Extracción de Metadatos (Prioridad: P2)

Como coleccionista que añade constantemente nuevos volúmenes a sus carpetas en disco, deseo que la aplicación detecte automáticamente la adición, modificación o eliminación de archivos de cómics sin necesidad de escaneos manuales forzados, extrayendo metadatos embebidos (como `ComicInfo.xml`) y generando miniaturas en segundo plano sin congelar la interfaz.

**Por qué esta prioridad**: Aumenta drásticamente la usabilidad y la escalabilidad del sistema cuando la biblioteca supera cientos o miles de volúmenes.

**Prueba Independiente**: Añadir un nuevo archivo `.cbz` a una carpeta vinculada y observar cómo la cuadrícula de la colección lo indexa y muestra su portada automáticamente en pocos segundos sin intervención manual.

**Escenarios de Aceptación**:
1. **Dado** que un directorio de colección está registrado en el sistema, **Cuando** se agrega un nuevo archivo `.cbz` o `.cbr` en el explorador de archivos, **Entonces** el *File Watcher* (patrón Observador) detecta el evento y notifica al trabajador de ingesta en segundo plano.
2. **Dado** que un cómic contiene un archivo de metadatos `ComicInfo.xml` en su interior, **Cuando** es procesado por el trabajador de ingesta, **Entonces** se extraen automáticamente título, número de entrega, año de publicación y saga para guardarlos en la base de datos relacional.
3. **Dado** que el cómic no tiene miniatura precargada, **Cuando** finaliza el escaneo, **Entonces** el adaptador genera y almacena la miniatura en la caché de persistencia sin retardar el hilo principal de la UI.

---

### Historia de Usuario 4 — Soporte de Protocolo Estándar OPDS y Descubrimiento mDNS (Prioridad: P3)

Como usuario que prefiere utilizar aplicaciones dedicadas de lectura en iPad o Android (como Panels, Chunky o Kuro Reader), deseo que el servidor anuncie su presencia en la red local mediante mDNS/Zeroconf y proporcione un feed estándar OPDS para importar o sincronizar catálogos directamente en mi aplicación lectora favorita.

**Por qué esta prioridad**: Extiende el ecosistema de la aplicación abriéndose a clientes estándar de la industria del cómic sin obligar al usuario a depender únicamente del navegador web.

**Prueba Independiente**: Abrir una app compatible con OPDS en una tableta en la misma Wi-Fi, verificar que el servidor aparezca automáticamente por mDNS y navegar por las colecciones a través del catálogo XML OPDS.

**Escenarios de Aceptación**:
1. **Dado** que el servidor de cómics está activo, **Cuando** un cliente local busca servicios mDNS, **Entonces** el servidor responde anunciando el servicio HTTP/OPDS local.
2. **Dado** que un cliente compatible con OPDS consulta `/opds`, **Cuando** envía su token de autorización, **Entonces** recibe un feed XML Atom conforme al estándar OPDS 1.2 / 2.0 listando colecciones y volúmenes.

---

### Casos Límite y Condiciones de Error

- **Archivo corrupto o incompleto**: Si un archivo `.cbz` o `.cbr` está dañado o truncado, el adaptador de lectura debe registrar un aviso mediante `tracing::warn!`, devolver una carátula o página de marcador de error y no provocar pánicos en el hilo ni en el servidor.
- **Acceso concurrente masivo**: Si múltiples clientes solicitan simultáneamente páginas de diferentes cómics, el pool de conexiones de base de datos y los adaptadores de archivos deben despachar las lecturas en tareas paralelas sin bloquearse mutuamente.
- **Pérdida de conexión de red local**: Si un dispositivo remoto pierde la señal Wi-Fi a mitad de lectura, el cliente web debe almacenar en caché la página actual y reintentar la conexión transparentemente al restablecer la red.
- **Formatos no soportados o carpetas vacías**: Archivos no reconocidos o subcarpetas de metadatos del SO (como `__MACOSX`) se omiten limpiamente durante la ingesta sin emitir falsos errores.

---

## Requisitos *(Obligatorio)*

### Requisitos Funcionales

- **FR-001**: El sistema DEBE organizarse como un **Monolito Modular** en `app/`, estructurado internamente en capas bien delimitadas: Dominio, Aplicación/Casos de Uso, Puertos/Interfaces e Infraestructura/Adaptadores.
- **FR-002**: El núcleo de Dominio DEBE estar 100% desacoplado de frameworks web, motores de GUI y librerías externas de persistencia, definiendo entidades puras (`Comic`, `Collection`, `ReadingProgress`, `Metadata`).
- **FR-003**: El sistema DEBE implementar el **Patrón Adaptador** mediante el trait común `ComicFileReader`, permitiendo la lectura intercambiable de `.cbz` (ZIP), `.cbr` (RAR), y sentando las bases para futuros adaptadores (`.pdf`, `.epub`).
- **FR-004**: El sistema DEBE proveer un **Patrón Fachada (`ComicFacade`)** que unifique la obtención de páginas, metadatos y miniaturas, encapsulando las operaciones complejas de descompresión y transformación.
- **FR-005**: El sistema DEBE implementar un **Proxy de Caché de Imágenes (`ImageCacheProxy`)** que gestione un almacenamiento en disco en `app/.cache/` para retener páginas y miniaturas procesadas bajo demanda, sirviendo de inmediato las ya existentes y aplicando políticas de tamaño máximo y limpieza periódica.
- **FR-006**: La capa de API DEBE mantener compatibilidad total con todos los endpoints REST actuales (`/api/collections`, `/api/comics/:id/cover`, `/api/comics/:id/page/:page`, etc.) y el middleware de seguridad por tokens.
- **FR-007**: El sistema DEBE incorporar un módulo de **Escaneo e Ingesta Asíncrono (`IngestionWorker`)** que utilice eventos del sistema operativo (`notify`) con canal Tokio `mpsc` y debouncing de 500 ms (patrón Observador) para detectar automáticamente altas, modificaciones o bajas de archivos en carpetas de colección, complementado con un escaneo inicial de verificación al arrancar.
- **FR-008**: El sistema DEBE implementar la extracción automática de metadatos desde archivos `ComicInfo.xml` cuando estén presentes dentro del cómic empaquetado.
- **FR-009**: La interfaz de escritorio en `iced` DEBE mantenerse como un puerto de entrada primario, comunicándose con la lógica de negocio mediante mensajes y comandos de The Elm Architecture (TEA) sin acceder directamente a I/O de disco.
- **FR-010**: El sistema DEBE implementar el **Patrón Estrategia** para gestionar algoritmos de redimensionado y modos de entrega de páginas según el cliente solicitante (escritorio vs. móvil vs. OPDS).
- **FR-011**: El servidor web local DEBE admitir streaming de páginas individuales redimensionadas bajo demanda para optimizar el ancho de banda y la memoria de clientes móviles.
- **FR-012**: El sistema DEBE registrar todos los eventos y errores mediante el ecosistema `tracing` con rotación diaria automática de logs en `app/log/`.
- **FR-013**: Toda la persistencia relacional DEBE residir en SQLite local con modo WAL y pool controlado mediante `sqlx`.
- **FR-014**: El sistema DEBE proveer un puerto de entrada OPDS 1.2 funcional (`/opds`) con catálogo XML Atom para navegación y adquisición en lectores móviles estándar, y DEBE definir el puerto y contrato de dominio para WebSockets de sincronización de progreso de lectura.
- **FR-015**: El sistema DEBE incluir descubrimiento automático de servicios en red local (mDNS/Zeroconf) para facilitar la vinculación sin introducir direcciones IP manualmente.

### Entidades Clave del Dominio

- **`Comic`**: Representa un volumen de cómic digital (identificador, título, ruta física, formato, año, número, saga, total de páginas, marca temporal de actualización).
- **`Collection`**: Agrupación lógica de cómics (identificador, nombre, icono visual opcional, fecha de creación).
- **`CollectionPath`**: Vínculo entre una colección y un directorio físico del sistema de archivos monitoreado.
- **`ComicMetadata`**: Metadatos bibliográficos detallados (autores, editorial, sinopsis, etiquetas, extraídos de `ComicInfo.xml` o ingresados manualmente).
- **`ReadingProgress`**: Estado de lectura por usuario/dispositivo (última página leída, porcentaje completado, fecha de última lectura).
- **`TrustedDevice`**: Dispositivo vinculado con autorización permanente (token alfanumérico, nombre de dispositivo, fecha de registro).

---

## Criterios de Éxito *(Obligatorio)*

### Resultados Medibles

- **SC-001**: 100% de las funcionalidades actuales de la aplicación (catálogo, lector desktop, visor de miniaturas, servidor HTTP, código QR y dispositivos de confianza) se mantienen operativas sin regresiones tras la reorganización modular.
- **SC-002**: La interfaz gráfica de usuario en escritorio responde en menos de 16 ms (manteniendo 60 FPS estables) durante la navegación y el escaneo en segundo plano, sin bloqueos del hilo principal.
- **SC-003**: La apertura y renderizado de una página de cómic previamente cacheada toma menos de 50 ms en clientes locales y remotos.
- **SC-004**: La adición de un nuevo formato de cómic (ej. PDF o EPUB) en el futuro requerirá únicamente la implementación del trait `ComicFileReader`, con 0 líneas de cambio en la capa de interfaz de usuario o en el servidor API.
- **SC-005**: El consumo de memoria RAM en reposo permanece por debajo de 120 MB independientemente de que la biblioteca contenga miles de cómics indexados.
- **SC-006**: Múltiples clientes remotos (al menos 4 dispositivos simultáneos) pueden leer páginas en paralelo sin degradación perceptible de velocidad (< 200 ms por página servida).

---

## Supuestos

- La aplicación seguirá ejecutándose de manera primordial en entornos de escritorio basados en Windows 11 / 10, con soporte compilable en Linux y macOS.
- La biblioteca de cómics física reside en discos locales o unidades de red accesibles directamente por el sistema de archivos del host.
- Los clientes remotos acceden exclusivamente a través de la red de área local (LAN o Wi-Fi doméstica), sin necesidad de apertura de puertos a internet público ni túneles externos.
- La base de datos SQLite embebida continuará utilizándose como almacén relacional primario por su ligereza, velocidad y cero configuración de instalación.
