import { defineStore } from "pinia";
import { shallowRef, markRaw } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { isWin } from "@/common/utils";

// Helper function to group files/folders Win11-style
export function groupFilesHelper(files, groupBy) {
  if (groupBy === "type") {
    const folders = [];
    const images = [];
    const videos = [];
    const others = [];
    for (const f of files) {
      const isDir =
        f.is_dir === true ||
        f.file_type === "directory" ||
        f.is_directory === true;
      if (isDir) {
        folders.push(f);
      } else if (
        f.file_type === 1 ||
        [
          "jpg",
          "jpeg",
          "png",
          "webp",
          "gif",
          "bmp",
          "tif",
          "tiff",
          "heic",
          "heif",
          "avif",
          "jxl",
        ].includes(f.extension?.toLowerCase())
      ) {
        images.push(f);
      } else if (
        f.file_type === 2 ||
        ["mp4", "mkv", "webm", "mov", "avi", "flv", "wmv"].includes(
          f.extension?.toLowerCase(),
        )
      ) {
        videos.push(f);
      } else {
        others.push(f);
      }
    }
    return [
      { title: "Папки", files: folders },
      { title: "Изображения", files: images },
      { title: "Видео", files: videos },
      { title: "Другие файлы", files: others },
    ].filter((g) => g.files.length > 0);
  }

  if (groupBy === "extension") {
    const groupsMap = {};
    for (const f of files) {
      const isDir =
        f.is_dir === true ||
        f.file_type === "directory" ||
        f.is_directory === true;
      const ext = isDir
        ? "Папка"
        : f.extension
          ? f.extension.toUpperCase()
          : "Без расширения";
      if (!groupsMap[ext]) {
        groupsMap[ext] = [];
      }
      groupsMap[ext].push(f);
    }
    const keys = Object.keys(groupsMap).sort((a, b) => {
      if (a === "Папка") return -1;
      if (b === "Папка") return 1;
      return a.localeCompare(b);
    });
    return keys.map((k) => ({
      title: k === "Папка" ? "Папки" : `Файлы ${k}`,
      files: groupsMap[k],
    }));
  }

  if (groupBy === "size") {
    const huge = []; // > 128 MB
    const large = []; // 1 MB - 128 MB
    const medium = []; // 100 KB - 1 MB
    const small = []; // < 100 KB
    const folders = [];

    for (const f of files) {
      const isDir =
        f.is_dir === true ||
        f.file_type === "directory" ||
        f.is_directory === true;
      if (isDir) {
        folders.push(f);
      } else {
        const size = f.size || 0;
        if (size >= 128 * 1024 * 1024) {
          huge.push(f);
        } else if (size >= 1 * 1024 * 1024) {
          large.push(f);
        } else if (size >= 100 * 1024) {
          medium.push(f);
        } else {
          small.push(f);
        }
      }
    }
    return [
      { title: "Папки", files: folders },
      { title: "Огромные (> 128 МБ)", files: huge },
      { title: "Крупные (1 МБ - 128 МБ)", files: large },
      { title: "Средние (100 КБ - 1 МБ)", files: medium },
      { title: "Маленькие (< 100 КБ)", files: small },
    ].filter((g) => g.files.length > 0);
  }

  if (groupBy === "date") {
    const last24h = [];
    const last12h = [];
    const thisWeek = [];
    const thisMonth = [];
    const folders = [];

    const now = Date.now();
    const H24 = 24 * 60 * 60 * 1000;
    const H12 = 12 * 60 * 60 * 1000;
    const startOf24h = now - H24;
    const startOf12h = now - H12;

    const d = new Date();
    const dayOfWeek = d.getDay();
    const daysToMonday = dayOfWeek === 0 ? 6 : dayOfWeek - 1;
    const startOfThisWeek = new Date(d.getFullYear(), d.getMonth(), d.getDate() - daysToMonday).getTime();
    const startOfThisMonth = new Date(d.getFullYear(), d.getMonth(), 1).getTime();

    for (const f of files) {
      const isDir =
        f.is_dir === true ||
        f.file_type === "directory" ||
        f.is_directory === true;
      if (isDir) {
        folders.push(f);
      } else {
        const time = f._modifiedTime || 0;
        if (time >= startOf12h) {
          last12h.push(f);
        } else if (time >= startOf24h) {
          last24h.push(f);
        } else if (time >= startOfThisWeek) {
          thisWeek.push(f);
        } else if (time >= startOfThisMonth) {
          thisMonth.push(f);
        } else {
          thisMonth.push(f);
        }
      }
    }

    return [
      { title: "Папки", files: folders },
      { title: "12 часов", files: last12h },
      { title: "24 часа", files: last24h },
      { title: "Неделя", files: thisWeek },
      { title: "Месяц", files: thisMonth },
    ].filter((g) => g.files.length > 0);
  }

  return [{ title: "Все файлы", files }];
}

/**
 * Store для галереи (Модуль 2)
 * Управляет отображением миниатюр, выделением, сортировками, историей действий
 */
export const useGalleryStore = defineStore("gallery", {
  state: () => ({
    // Настройки отображения
    thumbnailSize: 200,
    // Промежуток между миниатюрами внутри ряда (px, 0..50).
    // 0 = миниатюры притык друг к другу, 50 = максимальный зазор.
    thumbnailGap: 11,
    // shallowRef: большой массив файлов, нет смысла гонять Proxy по каждому элементу.
    // Обновляется через $patch / прямое присваивание, реактивность по длине/идентичности.
    files: [],
    isLoading: false,

    // Стек сравнения (выживает между папками)
    compareStack: [], // array of { path, name, extension, is_dir }

    // Счётчики обогащения (для индикатора в StatusBar)
    enrichingActive: 0, // currently in flight
    enrichingQueued: 0, // pending in queue

    // Группировка
    groupBy: "none", // none, type, date, size, extension

    // Сортировка
    sortBy: "name", // name, size, date, resolution, ai_source, model, loras
    sortOrder: "asc", // asc, desc
    folderSortSettings: {}, // { [folderPath]: { sortBy, sortOrder } }
    viewMode: "grid", // grid, list, table

    // Фильтры
    filters: {
      search: "",
      format: "", // png, jpg, webp, mp4, etc.
      minSize: 0,
      maxSize: 0,
      aiSource: "", // ComfyUI, Midjourney, etc.
      dateFrom: "",
      dateTo: "",
      model: "", // checkpoint / model filter
      lora: "", // LoRA filter
    },

    // Кеш AI-метаданных для сортировки по компонентам
    metadataCache: {}, // { [path]: AiMetadata }

    // Выделение (Command pattern)
    selectedIds: [], // массив путей выбранных файлов
    selectionHistory: [], // стек для Ctrl+Z / Ctrl+Shift+Z
    historyIndex: -1,

    // Масштабирование
    zoomLevel: 1,
    selectionMode: false,
    // Полноэкранный режим фокусировки: прячет весь UI кроме миниатюр
    // и узкой полосы избранного слева. Включается из режима выделения.
    focusMode: false,
    renamingPath: null,

    // Clipboard state
    clipboard: {
      action: null, // 'copy' | 'cut'
      paths: [], // array of file/folder paths
    },
    pasteProgress: {
      show: false,
      total: 0,
      current: 0,
      percentage: 0,
      action: null, // 'copy' | 'cut'
      cancelled: false,
      sourceDir: "",
      targetDir: "",
    },
    deletedHistory: [], // stack of arrays of TrashEntry
    isUndoing: false,
    trashItems: [],
  }),

  getters: {
    // Мемоизированный (через Vue computed) пайплайн отображения файлов.
    // Работает только при реальном изменении ссылок или фильтров/сортировки/группировки.
    displayedFiles: (state) => {
      const allFiles = Array.isArray(state.files) ? state.files : (state.files && Array.isArray(state.files.value) ? state.files.value : []);
      if (!allFiles || allFiles.length === 0) return [];

      const f = state.filters;
      const hasActiveFilter =
        !!f.format || !!f.aiSource || !!f.dateFrom || !!f.dateTo || !!f.search;
      const hasGrouping = !!state.groupBy && state.groupBy !== "none";
      const sortBy = state.sortBy;
      const sortOrder = state.sortOrder;

      let result = hasActiveFilter ? [...allFiles] : allFiles;

      if (hasActiveFilter) {
        const q = f.search ? String(f.search).toLowerCase() : null;
        const fromTime = f.dateFrom ? new Date(f.dateFrom).getTime() : null;
        const toTime = f.dateTo ? new Date(f.dateTo).getTime() : null;

        const normalizeSource = (v) =>
          v == null ? "" : String(v).trim().replace(/\s+/g, " ");
        const normalizeExt = (v) => {
          if (v == null) return "";
          return String(v).trim().toLowerCase();
        };

        result = result.filter((file) => {
          const fileExt = normalizeExt(file.extension);
          const wantExt = normalizeExt(f.format);
          if (wantExt && fileExt !== wantExt) return false;

          const fileSource = normalizeSource(file.ai_source);
          const wantSource = normalizeSource(f.aiSource);
          if (wantSource && fileSource !== wantSource) return false;

          const modifiedTime = file._modifiedTime ?? file.modified ?? 0;
          const modifiedMs =
            typeof modifiedTime === "number"
              ? modifiedTime
              : new Date(modifiedTime).getTime() || 0;

          if (fromTime != null && modifiedMs < fromTime) return false;
          if (toTime != null && modifiedMs > toTime) return false;

          if (q) {
            const nameLike =
              file.name ??
              file.base_name ??
              (file.path ? String(file.path).split(/[\\/]/).pop() : "") ??
              "";
            if (!String(nameLike).toLowerCase().includes(q)) return false;
          }

          return true;
        });
      }

      const isDefaultSort = sortBy === "name" && sortOrder === "asc";
      if (!isDefaultSort) {
        const dir = sortOrder === "desc" ? -1 : 1;
        result.sort((a, b) => {
          const aIsDir =
            a.is_dir === true ||
            a.file_type === "directory" ||
            a.is_directory === true;
          const bIsDir =
            b.is_dir === true ||
            b.file_type === "directory" ||
            b.is_directory === true;
          if (aIsDir && !bIsDir) return -1;
          if (!aIsDir && bIsDir) return 1;

          let cmp = 0;
          switch (sortBy) {
            case "size":
              cmp = (a.size || 0) - (b.size || 0);
              break;
            case "date":
              cmp = (a._modifiedTime || 0) - (b._modifiedTime || 0);
              break;
            case "resolution":
              cmp = (a.resolution?.width || 0) - (b.resolution?.width || 0);
              break;
            case "ai_source":
              cmp = (a.ai_source || "").localeCompare(b.ai_source || "");
              break;
            case "random":
              cmp = (a._randomWeight || 0) - (b._randomWeight || 0);
              break;
            case "name":
            default:
              cmp = (a.name || "").localeCompare(b.name || "");
              break;
          }
          return dir * (cmp || 0);
        });
      }

      if (hasGrouping) {
        const groups = groupFilesHelper(result, state.groupBy);
        const flat = [];
        for (const g of groups) {
          for (const item of g.files) flat.push(item);
        }
        return flat;
      }

      return result;
    },

    // Реактивные геттеры для стека сравнения (вместо методов → корректная зависимость в computed).
    compareStackSize: (state) => state.compareStack.length,
    canAddMoreToCompare: (state) => state.compareStack.length < 6,
    isPathInCompareStack: (state) => (path) => {
      if (!path) return false;
      for (const f of state.compareStack) if (f.path === path) return true;
      return false;
    },

    canUndo: (state) => state.historyIndex >= 0,
    canRedo: (state) => state.historyIndex < state.selectionHistory.length - 1,
  },

  actions: {
    cancelPaste() {
      this.pasteProgress.cancelled = true;
    },

    async fetchTrash() {
      this.trashItems = [];
    },

    setFiles(files) {
      const normalized = files.map((f) => ({
        ...f,
        _modifiedTime: f.modified ? new Date(f.modified).getTime() : 0,
        _randomWeight: Math.random(),
      }));
      this.files = normalized;
    },

    applyEnrichments(updates) {
      if (!Array.isArray(updates) || updates.length === 0) return;
      console.log("applyEnrichments updates received count:", updates.length, "Sample updates:", updates.slice(0, 3));
      const map = new Map(updates.map((u) => [u.path, u]));
      let changed = false;
      for (let i = 0; i < this.files.length; i++) {
        const entry = this.files[i];
        const u = map.get(entry.path);
        if (!u) continue;
        
        let entryChanged = false;
        const newEntry = { ...entry };
        
        if (u.dir_count != null && entry.dir_count !== u.dir_count) {
          newEntry.dir_count = u.dir_count;
          entryChanged = true;
        }
        if (u.file_count != null && entry.file_count !== u.file_count) {
          newEntry.file_count = u.file_count;
          entryChanged = true;
        }
        if (u.ai_source != null && entry.ai_source !== u.ai_source) {
          newEntry.ai_source = u.ai_source;
          entryChanged = true;
        }
        if (u.resolution != null) {
          if (!entry.resolution || entry.resolution.width !== u.resolution.width || entry.resolution.height !== u.resolution.height) {
            newEntry.resolution = u.resolution;
            entryChanged = true;
          }
        }
        
        if (!entry._enriched) {
          newEntry._enriched = true;
          entryChanged = true;
        }
        
        if (entryChanged) {
          this.files[i] = newEntry;
          changed = true;
        }
      }
      if (changed) {
        this.files = [...this.files];
      }
    },

    needsEnrichment(file) {
      if (!file || !file.path) return false;
      if (file._enriched) return false;
      const isDir =
        file.is_dir === true ||
        file.file_type === "directory" ||
        file.is_directory === true;
      if (isDir) {
        return file.dir_count == null || file.file_count == null;
      }
      const isImage = /^(png|jpe?g|webp|tiff?|avif|heic|heif|jxl|gif)$/i.test(
        file.extension || "",
      );
      if (!isImage) return false;
      return !file.resolution;
    },

    async requestEnrichments(extraPaths) {
      if (this._enrichmentInFlight) {
        this._enrichmentPending = this._enrichmentPending || new Set();
        for (const p of extraPaths) this._enrichmentPending.add(p);
        this.enrichingQueued = this._enrichmentPending.size;
        return;
      }
      this._enrichmentInFlight = new Set();
      this._enrichmentPending = new Set();
      for (const p of extraPaths) this._enrichmentPending.add(p);
      this.enrichingQueued = this._enrichmentPending.size;
      this.enrichingActive = this._enrichmentInFlight.size;

      while (this._enrichmentPending.size > 0) {
        const batch = Array.from(this._enrichmentPending).slice(0, 60);
        batch.forEach((p) => {
          this._enrichmentPending.delete(p);
          this._enrichmentInFlight.add(p);
        });
        this.enrichingQueued = this._enrichmentPending.size;
        this.enrichingActive = this._enrichmentInFlight.size;
        if (batch.length === 0) break;
        try {
          const updates = await invoke("enrich_entries", { paths: batch });
          this.applyEnrichments(updates);
        } catch (err) {
          console.warn("enrich_entries failed:", err);
        } finally {
          batch.forEach((p) => this._enrichmentInFlight.delete(p));
          this.enrichingActive = this._enrichmentInFlight.size;
        }
      }
      this._enrichmentInFlight = null;
      this._enrichmentPending = null;
      this.enrichingQueued = 0;
      this.enrichingActive = 0;
    },

    upsertFile(file, { pinToTop = false } = {}) {
      const normalized = {
        ...file,
        _modifiedTime: file.modified
          ? new Date(file.modified).getTime()
          : file._modifiedTime || Date.now(),
        _randomWeight: file._randomWeight !== undefined ? file._randomWeight : Math.random(),
      };
      const existingIndex = this.files.findIndex(
        (f) => f.path === normalized.path,
      );
      if (existingIndex >= 0) {
        this.files.splice(existingIndex, 1, normalized);
      } else if (pinToTop) {
        const firstNonDirIndex = this.files.findIndex(
          (f) =>
            !(
              f.is_dir === true ||
              f.file_type === "directory" ||
              f.is_directory === true
            ),
        );
        const insertIndex = firstNonDirIndex >= 0 ? 0 : this.files.length;
        this.files.splice(insertIndex, 0, normalized);
      } else {
        this.files.unshift(normalized);
      }
    },

    async setSorting(sortBy, order) {
      if (sortBy === "random") {
        this.reshuffleRandomWeights();
      }
      this.sortBy = sortBy;
      this.sortOrder = order;

      const { useNavigationStore } = await import("../navigation/store");
      const navigationStore = useNavigationStore();
      const currentPath = navigationStore.currentPath;
      if (currentPath) {
        if (!this.folderSortSettings) {
          this.folderSortSettings = {};
        }
        this.folderSortSettings[currentPath] = { sortBy, sortOrder: order };
      }
    },

    loadSortingForFolder(path) {
      if (!path) return;
      if (!this.folderSortSettings) {
        this.folderSortSettings = {};
      }
      const settings = this.folderSortSettings[path];
      if (settings) {
        this.sortBy = settings.sortBy || "name";
        this.sortOrder = settings.sortOrder || "asc";
      } else {
        this.sortBy = "name";
        this.sortOrder = "asc";
      }
    },

    reshuffleRandomWeights() {
      this.files.forEach((f) => {
        f._randomWeight = Math.random();
      });
      this.files = [...this.files];
    },

    setFilter(key, value) {
      this.filters[key] = value;
    },

    clearFilters() {
      this.filters = {
        search: "",
        format: "",
        minSize: 0,
        maxSize: 0,
        aiSource: "",
        dateFrom: "",
        dateTo: "",
      };
    },

    setZoom(level) {
      this.zoomLevel = Math.max(0.5, Math.min(5.12, level));
      this.thumbnailSize = Math.round(200 * this.zoomLevel);
    },
    setThumbnailGap(gapPx) {
      // 0..50 px. Округляем до целого, чтобы избежать дробных значений,
      // приводящих к артефактам суб-пиксельного рендеринга.
      const v = Math.round(Number(gapPx));
      this.thumbnailGap = Math.max(
        0,
        Math.min(50, Number.isFinite(v) ? v : 11),
      );
    },
    enterFocusMode() {
      // Доступно только из режима выделения — иначе игнорируем.
      if (!this.selectionMode) return;
      this.focusMode = true;
    },
    exitFocusMode() {
      this.focusMode = false;
    },
    toggleFocusMode() {
      if (this.focusMode) {
        this.focusMode = false;
      } else if (this.selectionMode) {
        this.focusMode = true;
      }
    },

    // --- Command pattern для выделения ---

    // Примечание: реальная реализация _pushSelectionState ниже, ближе к концу actions.

    primeSelectedForEnrichment() {
      const paths = [];
      for (const id of this.selectedIds) {
        const f = this.files.find((x) => x && x.path === id);
        if (f && this.needsEnrichment(f)) paths.push(f.path);
        if (paths.length >= 200) break;
      }
      if (paths.length > 0) this.requestEnrichments(paths);
    },

    // Compare stack (cross-folder) — использует getter isPathInCompareStack / canAddMoreToCompare

    isInCompareStack(path) {
      if (!path) return false;
      return this.isPathInCompareStack(path);
    },

    addToCompare(file) {
      if (!file || !file.path) return { ok: false, reason: "invalid" };
      const path = file.path;
      if (this.isPathInCompareStack(path))
        return { ok: false, reason: "duplicate" };
      if (this.compareStack.length >= 6) return { ok: false, reason: "max" };
      this.compareStack.push({
        path,
        name: file.name || path,
        extension: file.extension || null,
        is_dir: !!(
          file.is_dir === true ||
          file.file_type === "directory" ||
          file.is_directory === true
        ),
      });
      return { ok: true };
    },

    removeFromCompare(path) {
      if (!path) return;
      this.compareStack = this.compareStack.filter((f) => f.path !== path);
    },

    clearCompare() {
      this.compareStack = [];
    },

    // --- Реализация _pushSelectionState + selection history (должна быть одна) ---

    _pushSelectionState() {
      if (this.historyIndex < this.selectionHistory.length - 1) {
        this.selectionHistory = this.selectionHistory.slice(
          0,
          this.historyIndex + 1,
        );
      }
      this.selectionHistory.push([...this.selectedIds]);
      this.historyIndex = this.selectionHistory.length - 1;

      if (this.selectionHistory.length > 50) {
        this.selectionHistory.shift();
        this.historyIndex--;
      }
    },

    recordSelectionState() {
      // Публичный wrapper вокруг _pushSelectionState, совместимый со старым API.
      // Никаких параметров: используется текущий this.selectedIds. Старый код
      // передавал oldVal и делал двойное копирование, из-за чего undo-прыгало
      // на шаг вперед относительно реального состояния.
      this._pushSelectionState();
    },

    toggleSelection(filePath) {
      this._pushSelectionState();
      const index = this.selectedIds.indexOf(filePath);
      if (index >= 0) {
        this.selectedIds.splice(index, 1);
      } else {
        this.selectedIds.push(filePath);
      }
    },

    selectAll() {
      this._pushSelectionState();
      this.selectedIds = this.displayedFiles.map((f) => f.path);
    },

    clearSelection() {
      if (this.selectedIds.length > 0) {
        this._pushSelectionState();
        this.selectedIds = [];
      }
    },

    resetSelection() {
      this.selectedIds = [];
      this.selectionHistory = [];
      this.historyIndex = -1;
    },

    undo() {
      if (this.canUndo) {
        this.isUndoing = true;
        this.selectedIds = [...this.selectionHistory[this.historyIndex]];
        this.historyIndex--;
        this.isUndoing = false;
      }
    },

    redo() {
      if (this.canRedo) {
        this.isUndoing = true;
        this.historyIndex++;
        this.selectedIds = [...this.selectionHistory[this.historyIndex]];
        this.isUndoing = false;
      }
    },

    // --- Действия с файлами ---

    async deleteFiles(paths) {
      if (!paths || paths.length === 0) return;
      try {
        const result = await invoke("move_to_trash", { paths });
        if (result && result.length > 0) {
          this.deletedHistory.push(result);
        }
        this.files = this.files.filter((f) => !paths.includes(f.path));
        this.selectedIds = this.selectedIds.filter((id) => !paths.includes(id));
        await this.fetchTrash();
        return result;
      } catch (error) {
        console.error("Failed to delete files:", error);
        throw error;
      }
    },

    async deleteSelectedFiles() {
      const paths = [...this.selectedIds];
      const res = await this.deleteFiles(paths);
      this.selectedIds = [];
      return res;
    },

    async undoDelete() {
      if (this.deletedHistory.length === 0) return;
      const lastDeleted = this.deletedHistory.pop();
      if (!lastDeleted || lastDeleted.length === 0) return;

      const trashPaths = lastDeleted.map((entry) => entry.trashPath);
      try {
        await invoke("restore_from_trash", { trashPaths });
        const { useNavigationStore } = await import("../navigation/store");
        const navigationStore = useNavigationStore();
        await navigationStore.navigateTo(navigationStore.currentPath);
        this.setFiles(navigationStore.folders);
        await this.fetchTrash();
      } catch (error) {
        console.error("Failed to undo delete:", error);
        alert("Не удалось отменить удаление: " + error);
      }
    },

    async copySelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return;

      const firstSrc = this.selectedIds[0];
      const lastSlash = Math.max(firstSrc.lastIndexOf("\\"), firstSrc.lastIndexOf("/"));
      const sourceDir = lastSlash !== -1 ? firstSrc.substring(0, lastSlash) : "";

      this.pasteProgress.show = true;
      this.pasteProgress.total = this.selectedIds.length;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = "copy";
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = sourceDir;
      this.pasteProgress.targetDir = destPath;

      const sep = isWin ? "\\" : "/";
      let completedCount = 0;
      for (const src of this.selectedIds) {
        if (this.pasteProgress.cancelled) {
          break;
        }
        try {
          const fileName = src.split("\\").pop() || src.split("/").pop();
          await invoke("cross_copy", {
            src,
            dest: `${destPath}${sep}${fileName}`,
          });
        } catch (error) {
          console.error("Failed to copy:", error);
        } finally {
          completedCount++;
          this.pasteProgress.current = completedCount;
          this.pasteProgress.percentage = Math.round((completedCount / this.selectedIds.length) * 100);
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 600));
      this.pasteProgress.show = false;
      this.pasteProgress.total = 0;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = null;
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = "";
      this.pasteProgress.targetDir = "";
    },

    async moveSelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return;

      const firstSrc = this.selectedIds[0];
      const lastSlash = Math.max(firstSrc.lastIndexOf("\\"), firstSrc.lastIndexOf("/"));
      const sourceDir = lastSlash !== -1 ? firstSrc.substring(0, lastSlash) : "";

      this.pasteProgress.show = true;
      this.pasteProgress.total = this.selectedIds.length;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = "cut";
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = sourceDir;
      this.pasteProgress.targetDir = destPath;

      const sep = isWin ? "\\" : "/";
      let completedCount = 0;
      const movedPaths = [];
      for (const src of this.selectedIds) {
        if (this.pasteProgress.cancelled) {
          break;
        }
        try {
          const fileName = src.split("\\").pop() || src.split("/").pop();
          await invoke("cross_move", {
            src,
            dest: `${destPath}${sep}${fileName}`,
          });
          movedPaths.push(src);
        } catch (error) {
          console.error("Failed to move:", error);
        } finally {
          completedCount++;
          this.pasteProgress.current = completedCount;
          this.pasteProgress.percentage = Math.round((completedCount / this.selectedIds.length) * 100);
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 600));
      this.pasteProgress.show = false;
      this.pasteProgress.total = 0;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = null;
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = "";
      this.pasteProgress.targetDir = "";

      this.files = this.files.filter((f) => !movedPaths.includes(f.path));
      this.selectedIds = this.selectedIds.filter((id) => !movedPaths.includes(id));
    },

    setClipboard(action, paths) {
      this.clipboard.action = action;
      this.clipboard.paths = paths;
      try {
        invoke("write_to_system_clipboard", { paths });
      } catch (err) {
        console.warn("Failed to write to system clipboard:", err);
      }
    },

    clearClipboard() {
      this.clipboard.action = null;
      this.clipboard.paths = [];
    },

    async paste(destPath) {
      // 1. Try to read files from the system clipboard first
      let paths = [];
      let isSystemClipboard = false;
      try {
        paths = await invoke("read_from_system_clipboard");
        if (paths && paths.length > 0) {
          isSystemClipboard = true;
        }
      } catch (err) {
        console.warn("Failed to read files from system clipboard:", err);
      }

      // 2. Fallback to internal clipboard if system clipboard has no files
      let action = "copy";
      if (!isSystemClipboard) {
        if (!this.clipboard.action || this.clipboard.paths.length === 0) return;
        action = this.clipboard.action;
        paths = this.clipboard.paths;
      } else if (this.clipboard.action && this.clipboard.paths.length > 0) {
        // If system clipboard has the exact same paths as our internal clipboard,
        // use our internal clipboard's action (which could be 'cut').
        const systemSet = new Set(paths.map(p => p.replace(/\\/g, '/').toLowerCase()));
        const internalSet = new Set(this.clipboard.paths.map(p => p.replace(/\\/g, '/').toLowerCase()));
        const match = systemSet.size === internalSet.size && [...systemSet].every(p => internalSet.has(p));
        if (match) {
          action = this.clipboard.action;
          isSystemClipboard = false; // treat as internal to clear clipboard properly on cut
        }
      }

      const isCopy = action === "copy";
      
      const firstSrc = paths[0] || "";
      const lastSlash = Math.max(firstSrc.lastIndexOf("\\"), firstSrc.lastIndexOf("/"));
      const sourceDir = lastSlash !== -1 ? firstSrc.substring(0, lastSlash) : "";

      this.pasteProgress.show = true;
      this.pasteProgress.total = paths.length;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = action;
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = sourceDir;
      this.pasteProgress.targetDir = destPath;

      const sep = isWin ? "\\" : "/";
      let completedCount = 0;
      const movedPaths = [];
      for (const src of paths) {
        if (this.pasteProgress.cancelled) {
          break;
        }
        const fileName = src.split("\\").pop() || src.split("/").pop();
        const dest = `${destPath}${sep}${fileName}`;
        try {
          if (isCopy) {
            await invoke("cross_copy", { src, dest });
          } else if (action === "cut") {
            await invoke("cross_move", { src, dest });
            movedPaths.push(src);
          }
        } catch (error) {
          console.error(`Failed to ${action} ${src} to ${dest}:`, error);
        } finally {
          completedCount++;
          this.pasteProgress.current = completedCount;
          this.pasteProgress.percentage = Math.round((completedCount / paths.length) * 100);
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 600));
      this.pasteProgress.show = false;
      this.pasteProgress.total = 0;
      this.pasteProgress.current = 0;
      this.pasteProgress.percentage = 0;
      this.pasteProgress.action = null;
      this.pasteProgress.cancelled = false;
      this.pasteProgress.sourceDir = "";
      this.pasteProgress.targetDir = "";

      if (action === "cut" && !isSystemClipboard) {
        // Only clear paths that were actually cut-moved
        if (this.pasteProgress.cancelled) {
          this.clipboard.paths = this.clipboard.paths.filter((p) => !movedPaths.includes(p));
          if (this.clipboard.paths.length === 0) {
            this.clearClipboard();
          }
        } else {
          this.clearClipboard();
        }
      }

      const { useNavigationStore } = await import("../navigation/store");
      const navigationStore = useNavigationStore();
      await navigationStore.navigateTo(destPath);
      this.setFiles(navigationStore.folders);
    },

    // --- Работа с AI метаданными ---

    async getAiMetadata(filePath) {
      try {
        return await invoke("parse_ai_metadata", { path: filePath });
      } catch {
        return null;
      }
    },
  },
  persist: {
    paths: ["folderSortSettings", "viewMode"],
  },
});
