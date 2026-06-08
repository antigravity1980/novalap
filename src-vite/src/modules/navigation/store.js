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
    navigatedCount: 0,
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
      this.navigatedCount++
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

        // Начинаем отслеживание изменений
        await invoke('watch_directory', { path }).catch(err => console.error('Failed to watch directory:', err))

        this.selectedFiles = []
      } catch (error) {
        console.error('Navigation error:', error)
        if (typeof window !== 'undefined' && window.__tauri_ipc__) {
          this.folders = []
          this.selectedFiles = []
          throw error
        }
        
        // Browser fallback
        const suffix = path.endsWith('\\') ? '' : '\\';
        this.folders = [
          { name: 'Documents', path: path + suffix + 'Documents', is_dir: true, modified: new Date().toISOString(), size: 0 },
          { name: 'Downloads', path: path + suffix + 'Downloads', is_dir: true, modified: new Date().toISOString(), size: 0 },
          { name: 'Pictures', path: path + suffix + 'Pictures', is_dir: true, modified: new Date().toISOString(), size: 0 },
          { name: 'AI_Generations', path: path + suffix + 'AI_Generations', is_dir: true, modified: new Date().toISOString(), size: 0 },
          { name: 'comfyui_00124_.png', path: path + suffix + 'comfyui_00124_.png', is_dir: false, modified: new Date().toISOString(), size: 1048576, extension: 'png', ai_source: 'ComfyUI', resolution: { width: 1024, height: 1024 } },
          { name: 'midjourney_epic_sunset.jpg', path: path + suffix + 'midjourney_epic_sunset.jpg', is_dir: false, modified: new Date().toISOString(), size: 2048576, extension: 'jpg', ai_source: 'Midjourney', resolution: { width: 1440, height: 900 } },
          { name: 'stable_diffusion_robot.webp', path: path + suffix + 'stable_diffusion_robot.webp', is_dir: false, modified: new Date().toISOString(), size: 512000, extension: 'webp', ai_source: 'Stable Diffusion', resolution: { width: 512, height: 512 } },
          { name: 'test_video.mp4', path: path + suffix + 'test_video.mp4', is_dir: false, modified: new Date().toISOString(), size: 15485760, extension: 'mp4', ai_source: '', resolution: { width: 1920, height: 1080 } }
        ]
        
        this.treeFolders[path] = [
          { name: 'Documents', path: path + suffix + 'Documents', is_dir: true },
          { name: 'Downloads', path: path + suffix + 'Downloads', is_dir: true },
          { name: 'Pictures', path: path + suffix + 'Pictures', is_dir: true },
          { name: 'AI_Generations', path: path + suffix + 'AI_Generations', is_dir: true }
        ]
        
        this.selectedFiles = []
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
          await invoke('watch_directory', { path }).catch(err => console.error('Failed to watch directory:', err))
        } catch (error) {
          if (typeof window !== 'undefined' && window.__tauri_ipc__) {
            throw error
          }
          // Fallback
          const suffix = path.endsWith('\\') ? '' : '\\';
          this.folders = [
            { name: 'Documents', path: path + suffix + 'Documents', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'Downloads', path: path + suffix + 'Downloads', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'Pictures', path: path + suffix + 'Pictures', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'comfyui_00124_.png', path: path + suffix + 'comfyui_00124_.png', is_dir: false, modified: new Date().toISOString(), size: 1048576, extension: 'png', ai_source: 'ComfyUI', resolution: { width: 1024, height: 1024 } }
          ]
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
          await invoke('watch_directory', { path }).catch(err => console.error('Failed to watch directory:', err))
        } catch (error) {
          if (typeof window !== 'undefined' && window.__tauri_ipc__) {
            throw error
          }
          // Fallback
          const suffix = path.endsWith('\\') ? '' : '\\';
          this.folders = [
            { name: 'Documents', path: path + suffix + 'Documents', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'Downloads', path: path + suffix + 'Downloads', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'Pictures', path: path + suffix + 'Pictures', is_dir: true, modified: new Date().toISOString(), size: 0 },
            { name: 'comfyui_00124_.png', path: path + suffix + 'comfyui_00124_.png', is_dir: false, modified: new Date().toISOString(), size: 1048576, extension: 'png', ai_source: 'ComfyUI', resolution: { width: 1024, height: 1024 } }
          ]
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
        // Dev-mode fallback: one placeholder drive
        this.drives = [
          { name: 'Local Disk (C:)', path: 'C:\\', is_removable: false },
        ]
      }
    },

    async expandTreeFolder(path) {
      if (!this.treeFolders[path]) {
        try {
          const children = await invoke('expand_folder', { path })
          this.treeFolders[path] = children
        } catch (error) {
          console.error('Failed to expand folder tree:', error)
          if (typeof window !== 'undefined' && window.__tauri_ipc__) {
            throw error
          }
          const suffix = path.endsWith('\\') ? '' : '\\';
          this.treeFolders[path] = [
            { name: 'Documents', path: path + suffix + 'Documents', is_dir: true },
            { name: 'Downloads', path: path + suffix + 'Downloads', is_dir: true },
            { name: 'Pictures', path: path + suffix + 'Pictures', is_dir: true },
            { name: 'AI_Generations', path: path + suffix + 'AI_Generations', is_dir: true }
          ]
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
  persist: {
    paths: ['currentPath']
  }
})