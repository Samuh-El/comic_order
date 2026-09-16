//! Núcleo puro del dominio del sistema de cómics.
//!
//! Contiene las entidades puras, enumeraciones de errores y contratos de puertos.
//! Libre de dependencias externas a frameworks web o motores gráficos.

pub mod errors;
pub mod comic;
pub mod collection;
pub mod progress;
pub mod device;
pub mod ports;

pub use errors::{DomainError, ComicReaderError, RepositoryError};
pub use comic::{Comic, ComicFormat, ComicMetadata};
pub use collection::{Collection, CollectionPath};
pub use progress::ReadingProgress;
pub use device::TrustedDevice;
pub use ports::{
    ComicFileReader,
    ComicRepository,
    CollectionRepository,
    DeviceRepository,
    ProgressRepository,
    ImageCache,
    ReadingProgressNotifier,
};
