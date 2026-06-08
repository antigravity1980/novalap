import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Store для галереи (Модуль 2)
 * Управляет отображением миниатюр, выделением, сортировками, историей действий
 */
export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    // Настройки отображения
    thumbnailSize: 200,
    files: [],             // текущий список файлов для отображения
    filteredFiles: [],     // после применения фильтров/сортировки

    // Сортировка
    sortBy: 'name',        // name, size, date, resolution, ai_source, model, loras
    sortOrder: 'asc',      // asc, desc

    // Фильтры
    filters: {
      search: '',
      format: '',          // png, jpg, webp, mp4, etc.
      minSize: 0,
      maxSize: 0,
      aiSource: '',         // ComfyUI, Midjourney, etc.
      dateFrom: '',
      dateTo: '',
      model: '',            // checkpoint / model filter
      lora: '',             // LoRA filter
    },

    // Кеш AI-метаданных для сортировки по компонентам
    metadataCache: {},    // { [path]: AiMetadata }

    // Выделение (Command pattern)
    selectedIds: [],       // массив путей выбранных файлов
    selectionHistory: [],  // стек для Ctrl+Z / Ctrl+Shift+Z
    historyIndex: -1,

    // Масштабирование
    zoomLevel: 1,
    selectionMode: false,
    renamingPath: null,

    // Clipboard state
    clipboard: {
      action: null, // 'copy' | 'cut'
      paths: [],    // array of file/folder paths
    },
    deletedHistory: [], // stack of arrays of TrashEntry
    isUndoing: false,
  }),

  getters: {
    displayedFiles: (state) => {
      let files = [...state.files]

      // Фильтрация по формату
      if (state.filters.format) {
        files = files.filter(f => f.extension === state.filters.format)
      }

      // Фильтрация по AI-источнику
      if (state.filters.aiSource) {
        files = files.filter(f => f.ai_source === state.filters.aiSource)
      }

      // Фильтрация по дате
      if (state.filters.dateFrom) {
        const from = new Date(state.filters.dateFrom)
        files = files.filter(f => new Date(f.modified) >= from)
      }
      if (state.filters.dateTo) {
        const to = new Date(state.filters.dateTo)
        files = files.filter(f => new Date(f.modified) <= to)
      }

      // Поиск по имени
      if (state.filters.search) {
        const q = state.filters.search.toLowerCase()
        files = files.filter(f => f.name.toLowerCase().includes(q))
      }

      // Сортировка
      files.sort((a, b) => {
        // Папки всегда первыми
        const aIsDir = a.is_dir === true || a.file_type === 'directory' || a.is_directory === true
        const bIsDir = b.is_dir === true || b.file_type === 'directory' || b.is_directory === true
        if (aIsDir && !bIsDir) return -1
        if (!aIsDir && bIsDir) return 1

        let cmp = 0
        switch (state.sortBy) {
          case 'name':
            cmp = a.name.localeCompare(b.name)
            break
          case 'size':
            cmp = a.size - b.size
            break
          case 'date':
            cmp = new Date(a.modified) - new Date(b.modified)
            break
          case 'resolution':
            const ar = a.resolution?.width || 0
            const br = b.resolution?.width || 0
            cmp = ar - br
            break
          case 'ai_source':
            cmp = (a.ai_source || '').localeCompare(b.ai_source || '')
            break
        }
        return state.sortOrder === 'asc' ? cmp : -cmp
      })

      return files
    },

    canUndo: (state) => state.historyIndex >= 0,
    canRedo: (state) => state.historyIndex < state.selectionHistory.length - 1,
  },

  actions: {
    setFiles(files) {
      this.files = files
      this.filteredFiles = [...files]
    },

    setSorting(sortBy, order) {
      this.sortBy = sortBy
      this.sortOrder = order
    },

    setFilter(key, value) {
      this.filters[key] = value
    },

    clearFilters() {
      this.filters = {
        search: '',
        format: '',
        minSize: 0,
        maxSize: 0,
        aiSource: '',
        dateFrom: '',
        dateTo: '',
      }
    },

    setZoom(level) {
      this.zoomLevel = Math.max(0.5, Math.min(5.12, level))
      this.thumbnailSize = Math.round(200 * this.zoomLevel)
    },

    // --- Command pattern для выделения ---

    _pushSelectionState() {
      // Обрезаем будущие состояния при новом действии
      if (this.historyIndex < this.selectionHistory.length - 1) {
        this.selectionHistory = this.selectionHistory.slice(0, this.historyIndex + 1)
      }
      this.selectionHistory.push([...this.selectedIds])
      this.historyIndex = this.selectionHistory.length - 1

      // Ограничиваем историю 50 шагами
      if (this.selectionHistory.length > 50) {
        this.selectionHistory.shift()
        this.historyIndex--
      }
    },

    recordSelectionState(state) {
      if (this.historyIndex < this.selectionHistory.length - 1) {
        this.selectionHistory = this.selectionHistory.slice(0, this.historyIndex + 1)
      }
      this.selectionHistory.push([...state])
      this.historyIndex = this.selectionHistory.length - 1

      if (this.selectionHistory.length > 50) {
        this.selectionHistory.shift()
        this.historyIndex--
      }
    },

    toggleSelection(filePath) {
      this._pushSelectionState()
      const index = this.selectedIds.indexOf(filePath)
      if (index >= 0) {
        this.selectedIds.splice(index, 1)
      } else {
        this.selectedIds.push(filePath)
      }
    },

    selectAll() {
      this._pushSelectionState()
      this.selectedIds = this.displayedFiles.map(f => f.path)
    },

    clearSelection() {
      if (this.selectedIds.length > 0) {
        this._pushSelectionState()
        this.selectedIds = []
      }
    },

    undo() {
      if (this.canUndo) {
        this.isUndoing = true
        this.selectedIds = [...this.selectionHistory[this.historyIndex]]
        this.historyIndex--
        this.isUndoing = false
      }
    },

    redo() {
      if (this.canRedo) {
        this.isUndoing = true
        this.historyIndex++
        this.selectedIds = [...this.selectionHistory[this.historyIndex]]
        this.isUndoing = false
      }
    },

    // --- Действия с файлами ---

    async deleteFiles(paths) {
      if (!paths || paths.length === 0) return
      try {
        const result = await invoke('move_to_trash', { paths })
        if (result && result.length > 0) {
          this.deletedHistory.push(result)
        }
        // Remove from currently loaded files
        this.files = this.files.filter(f => !paths.includes(f.path))
        this.selectedIds = this.selectedIds.filter(id => !paths.includes(id))
        return result
      } catch (error) {
        console.error('Failed to delete files:', error)
        throw error
      }
    },

    async deleteSelectedFiles() {
      const paths = [...this.selectedIds]
      const res = await this.deleteFiles(paths)
      this.selectedIds = []
      return res
    },

    async undoDelete() {
      if (this.deletedHistory.length === 0) return
      const lastDeleted = this.deletedHistory.pop()
      if (!lastDeleted || lastDeleted.length === 0) return

      const trashPaths = lastDeleted.map(entry => entry.trashPath)
      try {
        await invoke('restore_from_trash', { trashPaths })
        const { useNavigationStore } = await import('../navigation/store')
        const navigationStore = useNavigationStore()
        await navigationStore.navigateTo(navigationStore.currentPath)
        this.setFiles(navigationStore.folders)
      } catch (error) {
        console.error('Failed to undo delete:', error)
        alert('Не удалось отменить удаление: ' + error)
      }
    },

    async copySelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return

      for (const src of this.selectedIds) {
        try {
          const fileName = src.split('\\').pop() || src.split('/').pop()
          await invoke('cross_copy', { src, dest: `${destPath}\\${fileName}` })
        } catch (error) {
          console.error('Failed to copy:', error)
        }
      }
    },

    async moveSelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return

      for (const src of this.selectedIds) {
        try {
          const fileName = src.split('\\').pop() || src.split('/').pop()
          await invoke('cross_move', { src, dest: `${destPath}\\${fileName}` })
        } catch (error) {
          console.error('Failed to move:', error)
        }
      }

      // Обновляем список
      this.files = this.files.filter(f => !this.selectedIds.includes(f.path))
      this.selectedIds = []
    },

    setClipboard(action, paths) {
      this.clipboard.action = action
      this.clipboard.paths = paths
    },

    clearClipboard() {
      this.clipboard.action = null
      this.clipboard.paths = []
    },

    async paste(destPath) {
      if (!this.clipboard.action || this.clipboard.paths.length === 0) return
      const { action, paths } = this.clipboard

      for (const src of paths) {
        const fileName = src.split('\\').pop() || src.split('/').pop()
        const dest = `${destPath}\\${fileName}`
        try {
          if (action === 'copy') {
            await invoke('cross_copy', { src, dest })
          } else if (action === 'cut') {
            await invoke('cross_move', { src, dest })
          }
        } catch (error) {
          console.error(`Failed to ${action} ${src} to ${dest}:`, error)
        }
      }

      if (action === 'cut') {
        this.clearClipboard()
      }

      // Refresh current directory
      const { useNavigationStore } = await import('../navigation/store')
      const navigationStore = useNavigationStore()
      await navigationStore.navigateTo(destPath)
      this.setFiles(navigationStore.folders)
    },

    // --- Работа с AI метаданными ---

    async getAiMetadata(filePath) {
      try {
        return await invoke('parse_ai_metadata', { path: filePath })
      } catch {
        return null
      }
    },
  },
})