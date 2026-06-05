/**
 * Module 1: Navigation & File Manager
 *
 * Реализует прямой доступ к файловой системе, дерево папок,
 * кросс-дисковые операции.
 */
pub mod explorer;
pub mod tree;
pub mod operations;

// Реэкспорт всех Tauri-команд для регистрации в main.rs
pub use explorer::*;
pub use tree::*;
pub use operations::*;