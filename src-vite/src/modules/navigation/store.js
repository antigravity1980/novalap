import { useGalleryStore } from "@/modules/gallery/store";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { clearThumbnailCache } from "@/modules/gallery/explorerThumbnailsCache";

/**
 * Store для навигации (Модуль 1)
 * Управляет выбранной директорией, историей навигации, дисками
 */
export const useNavigationStore = defineStore("navigation", {
  // eslint-disable-next-line no-unused-vars
  // normalize paths to avoid treeFolders key mismatch ("/" vs "\\")

  state: () => ({
    // NOTE: normalize paths to avoid treeFolders key mismatch ("/" vs "\\")
    // across backend/frontend calls.
    _pathNormMode: "backslash",
    currentPath: "",
    history: [],
    historyIndex: -1,
    drives: [],
    folders: [], // содержимое текущей папки (файлы + директории)
    treeFolders: {}, // кеш поддерева папок { path: TreeFolder[] }
    isLoading: false,
    selectedFiles: [], // выбранные файлы в текущей директории
    navigatedCount: 0,
    pendingTreeRenamePath: "",
    watchUnlisten: null, // unlisten fn для watch_directory (Tauri)
  }),

  getters: {
    canGoBack: (state) => state.historyIndex > 0,
    canGoForward: (state) => state.historyIndex < state.history.length - 1,
    currentFolderFiles: (state) => state.folders.filter((f) => !f.is_dir),
    currentFolderDirs: (state) => state.folders.filter((f) => f.is_dir),
    parentFolders: (state) => {
      if (!state.currentPath) return [];
      return state.currentPath
        .split("\\")
        .filter(Boolean)
        .map((part, i, arr) => ({
          name: part,
          path: arr.slice(0, i + 1).join("\\"),
        }));
    },
  },

  actions: {
    normalizePath(path) {
      if (!path) return "";
      // unify separators to backslash, trim trailing separators (except root like C:\)
      let p = String(path).replace(/[\/]+/g, "\\");
      // remove trailing slash unless it is like "C:\\" or "\\\\server\\share\\"
      p = p.replace(/\\+$/, "");
      // restore drive root "C:\" if it got trimmed
      if (/^[a-zA-Z]:\\?$/.test(p)) p = p.replace(/\\?$/, "\\");
      return p;
    },
    async _stopWatching() {
      try {
        if (typeof this.watchUnlisten === "function") {
          this.watchUnlisten();
        }
      } catch (e) {
        console.warn("Failed to unwatch directory:", e);
      } finally {
        this.watchUnlisten = null;
      }
    },

    async _startWatching(path) {
      // Точка invoke watch_directory должна вернуть unlisten fn (если реализация такова).
      // Если бэкенд возвращает void, мы просто логируем.
      try {
        const unlisten = await invoke("watch_directory", { path });
        if (typeof unlisten === "function") {
          this.watchUnlisten = unlisten;
        }
      } catch (err) {
        console.error("Failed to watch directory:", err);
      }
    },

    async navigateTo(path) {
      path = this.normalizePath(path);
      clearThumbnailCache();
      this.navigatedCount++;
      this.isLoading = true;
      try {
        // Добавляем в историю
        if (
          this.historyIndex >= 0 &&
          this.history[this.historyIndex] !== path
        ) {
          this.history = this.history.slice(0, this.historyIndex + 1);
        }
        this.history.push(path);
        this.historyIndex = this.history.length - 1;
        this.currentPath = path;

        // Загружаем содержимое
        this.folders = await invoke("list_directory", { path });
        const galleryStore = useGalleryStore();
        galleryStore.setFiles(this.folders);

        // Кешируем поддерево для tree view
        const treeData = await invoke("expand_folder", { path });
        this.treeFolders[path] = treeData;

        // Перезапускаем наблюдение (убираем дубликаты подписчиков)
        await this._stopWatching();
        await this._startWatching(path);

        this.selectedFiles = [];
      } catch (error) {
        console.error("Navigation error:", error);
        this.folders = [];
        this.selectedFiles = [];
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async goBack() {
      if (this.canGoBack) {
        this.historyIndex--;
        const path = this.history[this.historyIndex];
        this.currentPath = path;
        this.isLoading = true;
        try {
          this.folders = await invoke("list_directory", { path });
          const galleryStore = useGalleryStore();
          galleryStore.setFiles(this.folders);

          await this._stopWatching();
          await this._startWatching(path);
        } catch (error) {
          this.folders = [];
          throw error;
        } finally {
          this.isLoading = false;
        }
      }
    },

    async goForward() {
      if (this.canGoForward) {
        this.historyIndex++;
        const path = this.history[this.historyIndex];
        this.currentPath = path;
        this.isLoading = true;
        try {
          this.folders = await invoke("list_directory", { path });
          const galleryStore = useGalleryStore();
          galleryStore.setFiles(this.folders);

          await this._stopWatching();
          await this._startWatching(path);
        } catch (error) {
          this.folders = [];
          throw error;
        } finally {
          this.isLoading = false;
        }
      }
    },

    async loadDrives() {
      try {
        this.drives = await invoke("get_drives");
      } catch (error) {
        console.error("Failed to load drives:", error);
        this.drives = [];
      }
    },

    async expandTreeFolder(path) {
      path = this.normalizePath(path);

      if (!this.treeFolders[path]) {
        try {
          const children = await invoke("expand_folder", { path });
          this.treeFolders[path] = children;
        } catch (error) {
          console.error("Failed to expand folder tree:", error);
          this.treeFolders[path] = [];
          throw error;
        }
      }
    },

    /** Всегда перезагружает дерево для указанного пути (даже если уже закешировано) */
    async refreshTreeFolder(path) {
      path = this.normalizePath(path);

      try {
        const children = await invoke("expand_folder", { path });
        this.treeFolders[path] = children;
      } catch (error) {
        console.error("Failed to refresh tree folder:", error);
        if (typeof window !== "undefined" && window.__tauri_ipc__) {
          throw error;
        }
      }
    },

    async refresh() {
      if (this.currentPath) {
        await this.navigateTo(this.currentPath);
      }
    },

    toggleFileSelection(filePath) {
      const index = this.selectedFiles.indexOf(filePath);
      if (index >= 0) {
        this.selectedFiles.splice(index, 1);
      } else {
        this.selectedFiles.push(filePath);
      }
    },

    clearSelection() {
      this.selectedFiles = [];
    },

    selectAll() {
      this.selectedFiles = this.folders
        .filter((f) => !f.is_dir)
        .map((f) => f.path);
    },
  },
  persist: {
    paths: ["currentPath"],
  },
});
