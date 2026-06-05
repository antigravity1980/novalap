/**
 * Module 4: File Operations & AI Parsing (Backend)
 *
 * Содержит:
 * - ai_parser — парсинг AI-метаданных из PNG, WebP, MP4
 * - batch — массовые трансформации (resize, format, rename)
 * - stripping — удаление метаданных
 * - trash — .ams_trash система undo
 */
pub mod ai_parser;
pub mod batch;
pub mod stripping;
pub mod trash;
pub mod color_correction;
pub mod optimizers;

pub use ai_parser::*;
pub use batch::*;
pub use stripping::*;
pub use trash::*;
pub use color_correction::*;
pub use optimizers::*;
