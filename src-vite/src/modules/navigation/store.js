import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Store для навигации (Модуль 1)
 * Управляет выбранной директорией, историей навигации, дисками
 */
export const useNavigationStore = defineStore('navigation', {
  state: () => ({
    currentPath: '',
    history: [],
    historyIndex: -1,
    drives: [],
    folders: [],         // содержимое текущей папки (файлы + директории)
    treeFolders: {},     // кеш поддерева папок { path: TreeFolder[] }
    isLoading: false,
    selectedFiles: [],   // выбранные файлы в текущей директории
  }),

  getters: {
    canGoBack: (state) => state.historyIndex > 0,
    canGoForward: (state) => state.historyIndex < state.history.length - 1,
    currentFolderFiles: (state) => state.folders.filter(f => !f.is_dir),
    currentFolderDirs: (state) => state.folders.filter(f => f.is_dir),
    parentFolders: (state) => {
      if (!state.currentPath) return []
      return state.currentPath.split('\\').filter(Boolean).map((part, i, arr) => ({
        name: part,
        path: arr.slice(0, i + 1).join('\\')
      }))
    },
  },

  actions: {
    async navigateTo(path) {
      this.isLoading = true
      try {
        // Добавляем в историю
        if (this.historyIndex >= 0 && this.history[this.historyIndex] !== path) {
          this.history = this.history.slice(0, this.historyIndex + 1)
        }
        this.history.push(path)
        this.historyIndex = this.history.length - 1
        this.currentPath = path

        // Загружаем содержимое
        this.folders = await invoke('list_directory', { path })

        // Кешируем поддерево для tree view
        const treeData = await invoke('expand_folder', { path })
        this.treeFolders[path] = treeData

        this.selectedFiles = []
      } catch (error) {
        console.error('Navigation error:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async goBack() {
      if (this.canGoBack) {
        this.historyIndex--
        const path = this.history[this.historyIndex]
        this.currentPath = path
        this.isLoading = true
        try {
          this.folders = await invoke('list_directory', { path })
        } finally {
          this.isLoading = false
        }
      }
    },

    async goForward() {
      if (this.canGoForward) {
        this.historyIndex++
        const path = this.history[this.historyIndex]
        this.currentPath = path
        this.isLoading = true
        try {
          this.folders = await invoke('list_directory', { path })
        } finally {
          this.isLoading = false
        }
      }
    },

    async loadDrives() {
      try {
        this.drives = await invoke('get_drives')
      } catch (error) {
        console.error('Failed to load drives:', error)
      }
    },

    async expandTreeFolder(path) {
      if (!this.treeFolders[path]) {
        try {
          const children = await invoke('expand_folder', { path })
          this.treeFolders[path] = children
        } catch (error) {
          console.error('Failed to expand folder tree:', error)
          this.treeFolders[path] = []
        }
      }
    },

    async refresh() {
      if (this.currentPath) {
        await this.navigateTo(this.currentPath)
      }
    },

    toggleFileSelection(filePath) {
      const index = this.selectedFiles.indexOf(filePath)
      if (index >= 0) {
        this.selectedFiles.splice(index, 1)
      } else {
        this.selectedFiles.push(filePath)
      }
    },

    clearSelection() {
      this.selectedFiles = []
    },

    selectAll() {
      this.selectedFiles = this.folders
        .filter(f => !f.is_dir)
        .map(f => f.path)
    },
  },
})