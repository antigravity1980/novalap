# LapAI Architecture

## Структура модулей

### Backend (Rust) — `src-tauri/src/`

```
src-tauri/src/
├── main.rs                    # Точка входа с регистрацией модулей
├── lib.rs                     # Общая библиотека (если нужно)
├── t_common.rs                # Общие типы и константы (существующий)
├── t_config.rs                # Конфигурация (существующий)
├── t_storage.rs               # Хранилище (существующий)
├── t_utils.rs                 # Утилиты (существующий)
├── t_sqlite.rs                # База данных (существующий)
│
├── module1_navigation/        # Модуль 1: Навигация и Файловый Менеджер
│   ├── mod.rs                 # Публичные функции и реэкспорты
│   ├── explorer.rs            # Чтение файловой системы «на лету»
│   ├── tree.rs                # Дерево папок/дисков
│   └── operations.rs          # Кросс-дисковые copy/move
│
├── module2_gallery/           # Модуль 2: (частично на фронтенде)
│   └── mod.rs                 # Вспомогательные Rust-функции для галереи
│
├── module4_backend/           # Модуль 4: Файловые Операции и AI-Парсинг
│   ├── mod.rs
│   ├── ai_parser.rs           # Парсер AI-метаданных (PNG tEXt/iTXt, WebP, MP4)
│   ├── batch.rs               # Batch processing (resize, format, rename)
│   ├── stripping.rs           # Удаление метаданных
│   ├── color_correction.rs    # Цветокоррекция
│   ├── optimizers.rs          # Интеграция с pngquant, mozjpeg
│   └── trash.rs               # .ams_trash — undo-логика
│
├── t_cmds.rs                  # Tauri-команды (дополняется новыми)
├── t_image.rs                 # Работа с изображениями (существующий)
├── t_video.rs                 # Работа с видео (существующий)
├── t_ai.rs                    # AI Engine (существующий)
├── t_face.rs                  # Face Recognition (существующий)
├── ... (остальные существующие)
```

### Frontend (Vue 3) — `src-vite/src/`

```
src-vite/src/
├── main.js
├── App.vue                    # (существующий, дорабатывается)
│
├── modules/                   # Модульная архитектура фронтенда
│   ├── navigation/            # Модуль 1: Навигация
│   │   ├── components/        # Компоненты навигации
│   │   │   ├── ExplorerTree.vue      # Дерево папок
│   │   │   ├── DriveSelector.vue     # Выбор диска
│   │   │   └── TabView.vue           # Многооконный режим (вкладки)
│   │   └── store.js           # Store для навигации (Pinia)
│   │
│   ├── gallery/               # Модуль 2: Галерея
│   │   ├── components/
│   │   │   ├── GalleryGrid.vue       # Основная сетка миниатюр
│   │   │   ├── ThumbnailCard.vue     # Карточка миниатюры с инфо
│   │   │   ├── ZoomSlider.vue        # Ползунок масштабирования
│   │   │   └── FilterBar.vue         # Панель сортировок/фильтров
│   │   └── store.js           # Store для галереи (выделение, история)
│   │
│   ├── viewer/                # Модуль 3: Визуализация
│   │   ├── components/
│   │   │   ├── QuickLook.vue         # Быстрый просмотр (overlay)
│   │   │   ├── PromptViewer.vue      # Инспектор промптов
│   │   │   ├── CompareView.vue       # Режим сравнения
│   │   │   └── QuickCrop.vue         # Быстрый кроп
│   │   └── store.js
│   │
│   └── operations/            # Модуль 4: (UI для batch-операций)
│       └── components/
│           ├── BatchResize.vue
│           ├── BatchConvert.vue
│           ├── BatchRename.vue
│           └── BatchProgress.vue
│
├── stores/                    # Существующие Pinia stores (дорабатываются)
│   ├── configStore.js
│   ├── libraryStore.js
│   └── uiStore.js
│
├── components/                # Существующие компоненты (дорабатываются)
├── composables/
├── common/
└── views/
```

## API-контракты (Tauri-команды)

### Модуль 1: Навигация
- `list_directory(path: String) -> Vec<FileEntry>` — список файлов в директории
- `get_drives() -> Vec<DriveInfo>` — список доступных дисков (Windows) / mount points
- `get_tree(path: String) -> TreeFolder` — поддерево папок
- `cross_copy(src: String, dest: String) -> Result` — копирование между дисками
- `cross_move(src: String, dest: String) -> Result` — перемещение между дисками
- `open_in_explorer(path: String) -> Result` — открыть в системном проводнике

### Модуль 2: Галерея (в основном фронтенд)
- `get_file_resolution(path: String) -> (u32, u32)` — получение разрешения файла
- `get_ai_source(path: String) -> String` — определение источника генерации

### Модуль 4: Backend
- `parse_ai_metadata(path: String) -> AiMetadata` — парсинг AI-метаданных
- `batch_resize(files: Vec<String>, preset: ResizePreset) -> BatchResult`
- `batch_convert(files: Vec<String>, format: String, quality: u8) -> BatchResult`
- `batch_rename(files: Vec<String>, mask: String, counter_start: u32) -> BatchResult`
- `strip_metadata(files: Vec<String>) -> BatchResult`
- `batch_color_correct(files: Vec<String>, saturation: f32, gamma: f32) -> BatchResult`
- `optimize_with_pngquant(files: Vec<String>) -> BatchResult`
- `optimize_with_mozjpeg(files: Vec<String>) -> BatchResult`
- `move_to_trash(paths: Vec<String>) -> Result` — перенос в .ams_trash
- `restore_from_trash(paths: Vec<String>) -> Result` — восстановление из .ams_trash
- `get_trash_contents() -> Vec<TrashEntry>` — список файлов в корзине

## Типы данных

```typescript
// FileEntry (фронтенд)
interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string;  // ISO date
  resolution?: { width: number; height: number };
  ai_source?: string;
}

// AiMetadata (фронтенд)
interface AiMetadata {
  positive_prompt?: string;
  negative_prompt?: string;
  seed?: number;
  steps?: number;
  cfg_scale?: number;
  model?: string;
  loras?: string[];
  workflow?: string;  // ComfyUI workflow JSON
  source_engine?: string;  // ComfyUI, Midjourney, etc.
}

// TrashEntry
interface TrashEntry {
  original_path: string;
  trash_path: string;
  deleted_at: string;
  size: number;
}