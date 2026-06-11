import { defineStore } from "pinia";
import { shallowRef, markRaw } from "vue";
import { invoke } from "@tauri-apps/api/core";

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
    const today = [];
    const yesterday = [];
    const thisWeek = [];
    const thisMonth = [];
    const lastMonth = [];
    const thisYear = [];
    const older = [];
    const folders = [];

    const now = new Date();
    const startOfToday = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate(),
    ).getTime();
    const startOfYesterday = startOfToday - 24 * 60 * 60 * 1000;
    const dayOfWeek = now.getDay();
    const daysToMonday = dayOfWeek === 0 ? 6 : dayOfWeek - 1;
    const startOfThisWeek = startOfToday - daysToMonday * 24 * 60 * 60 * 1000;
    const startOfThisMonth = new Date(
      now.getFullYear(),
      now.getMonth(),
      1,
    ).getTime();
    const startOfLastMonth = new Date(
      now.getFullYear(),
      now.getMonth() - 1,
      1,
    ).getTime();
    const startOfThisYear = new Date(now.getFullYear(), 0, 1).getTime();

    for (const f of files) {
      const isDir =
        f.is_dir === true ||
        f.file_type === "directory" ||
        f.is_directory === true;
      if (isDir) {
        folders.push(f);
      } else {
        const time = f._modifiedTime || 0;
        if (time >= startOfToday) {
          today.push(f);
        } else if (time >= startOfYesterday) {
          yesterday.push(f);
        } else if (time >= startOfThisWeek) {
          thisWeek.push(f);
        } else if (time >= startOfThisMonth) {
          thisMonth.push(f);
        } else if (time >= startOfLastMonth) {
          lastMonth.push(f);
        } else if (time >= startOfThisYear) {
          thisYear.push(f);
        } else {
          older.push(f);
        }
      }
    }

    return [
      { title: "Папки", files: folders },
      { title: "Сегодня", files: today },
      { title: "Вчера", files: yesterday },
      { title: "Ранее на этой неделе", files: thisWeek },
      { title: "Ранее в этом месяце", files: thisMonth },
      { title: "В прошлом месяце", files: lastMonth },
      { title: "Ранее в этом году", files: thisYear },
      { title: "Давно", files: older },
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
    filteredFiles: [],
    isLoading: false,
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
    deletedHistory: [], // stack of arrays of TrashEntry
    isUndoing: false,
    // Initialize with cached count so the badge is correct before first fetch
    trashItems: (() => {
      try {
        const cached = localStorage.getItem("lapai_trash_cache");
        return cached ? JSON.parse(cached) : [];
      } catch {
        return [];
      }
    })(),
  }),

  getters: {
    // Мемоизированный (через Vue computed) пайплайн отображения файлов.
    // Работает только при реальном изменении ссылок или фильтров/сортировки/группировки.
    displayedFiles: (state) => {
      // alias для читаемости + одиночная точка входа
      const allFiles = state.files ? state.files : (state.files || []);
      if (!allFiles || allFiles.length === 0) return [];

      const f = state.filters;
      const hasActiveFilter =
        !!f.format || !!f.aiSource || !!f.dateFrom || !!f.dateTo || !!f.search;
      const hasGrouping = !!state.groupBy && state.groupBy !== "none";

      // Сортировка
      const sortBy = state.sortBy;
      const sortOrder = state.sortOrder;
      // Используем копию только когда нужна сортировка/фильтрация
      let result = [...allFiles];

      if (hasActiveFilter) {
        const q = f.search ? f.search.toLowerCase() : null;
        const fromTime = f.dateFrom ? new Date(f.dateFrom).getTime() : null;
        const toTime = f.dateTo ? new Date(f.dateTo).getTime() : null;
        result = result.filter(file => {
          if (state.filters.format && file.extension !== state.filters.format) return false;
          if (state.filters.aiSource && file.ai_source !== state.filters.aiSource) return false;
          if (fromTime != null && (f._modifiedTime || 0) < fromTime)
            return false;
          if (toTime != null && (f._modifiedTime || 0) > toTime) return false;
          if (q && !f.name.toLowerCase().includes(q)) return false;
          return true;
        });
      }

      // Сортировка по выбранному полю
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
          case "name":
          default:
            cmp = (a.name || "").localeCompare(b.name || "");
            break;
        }
        return dir * (cmp || 0);
      });

      if (hasGrouping) {
        // groupFilesHelper возвращает массив групп, разворачиваем в плоский список
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
    async fetchTrash() {
      try {
        this.trashItems = await invoke("get_trash_contents");
        try {
          localStorage.setItem(
            "lapai_trash_cache",
            JSON.stringify(this.trashItems),
          );
        } catch {}
      } catch (err) {
        console.error("Failed to load trash:", err);
      }
    },

    setFiles(files) {
      const normalized = files.map((f) => ({
        ...f,
        _modifiedTime: f.modified ? new Date(f.modified).getTime() : 0,
      }));
      // shallowRef заменяем напрямую — реактивность по identity.
      this.files = normalized;
      this.filteredFiles = [...normalized];
    },

    applyEnrichments(updates) {
      if (!Array.isArray(updates) || updates.length === 0) return;
      const map = new Map(updates.map((u) => [u.path, u]));
      let changed = false;
      for (const entry of this.files) {
        const u = map.get(entry.path);
        if (!u) continue;
        if (u.dir_count != null && entry.dir_count !== u.dir_count) {
          entry.dir_count = u.dir_count;
          changed = true;
        }
        if (u.file_count != null && entry.file_count !== u.file_count) {
          entry.file_count = u.file_count;
          changed = true;
        }
        if (u.ai_source != null && entry.ai_source !== u.ai_source) {
          entry.ai_source = u.ai_source;
          changed = true;
        }
      }
      if (changed) {
        this.filteredFiles = [...this.files];
      }
    },

    needsEnrichment(file) {
      if (!file || !file.path) return false;
      const isDir =
        file.is_dir === true ||
        file.file_type === "directory" ||
        file.is_directory === true;
      if (isDir) {
        return !file.dir_count || !file.file_count;
      }
      return (
        !file.ai_source &&
        /\.(png|jpe?g|webp|tiff?|avif|heic|heif|jxl|gif)$/i.test(
          file.extension || "",
        )
      );
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
      this.filteredFiles = [...this.files];
    },

    setSorting(sortBy, order) {
      this.sortBy = sortBy;
      this.sortOrder = order;
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
      const promises = this.selectedIds.map(async (src) => {
        try {
          const fileName = src.split("\\").pop() || src.split("/").pop();
          await invoke("cross_copy", {
            src,
            dest: `${destPath}\\${fileName}`,
          });
        } catch (error) {
          console.error("Failed to copy:", error);
        }
      });
      await Promise.all(promises);
    },

    async moveSelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return;
      const promises = this.selectedIds.map(async (src) => {
        try {
          const fileName = src.split("\\").pop() || src.split("/").pop();
          await invoke("cross_move", {
            src,
            dest: `${destPath}\\${fileName}`,
          });
        } catch (error) {
          console.error("Failed to move:", error);
        }
      });
      await Promise.all(promises);
      this.files = this.files.filter((f) => !this.selectedIds.includes(f.path));
      this.selectedIds = [];
    },

    setClipboard(action, paths) {
      this.clipboard.action = action;
      this.clipboard.paths = paths;
    },

    clearClipboard() {
      this.clipboard.action = null;
      this.clipboard.paths = [];
    },

    async paste(destPath) {
      if (!this.clipboard.action || this.clipboard.paths.length === 0) return;
      const { action, paths } = this.clipboard;
      const promises = paths.map(async (src) => {
        const fileName = src.split("\\").pop() || src.split("/").pop();
        const dest = `${destPath}\\${fileName}`;
        try {
          if (action === "copy") {
            await invoke("cross_copy", { src, dest });
          } else if (action === "cut") {
            await invoke("cross_move", { src, dest });
          }
        } catch (error) {
          console.error(`Failed to ${action} ${src} to ${dest}:`, error);
        }
      });
      await Promise.all(promises);

      if (action === "cut") {
        this.clearClipboard();
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
});
