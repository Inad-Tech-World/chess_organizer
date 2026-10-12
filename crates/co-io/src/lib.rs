//! Import and export: TRF, FIDE and national rating lists, other programs'
//! exports, and printable lists rendered to HTML, PDF, CSV and XLSX.

pub mod fide_list;
pub mod national_list;
pub mod render;
pub mod report;
pub mod swiss_manager_import;
pub mod trf;

pub use report::{ListKind, Report};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] co_core::Error),
    #[error(transparent)]
    Trf(#[from] trf::TrfError),
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
