import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

// Helper function to group files/folders Win11-style
export function groupFilesHelper(files, groupBy) {
  if (groupBy === 'type') {
    const folders = []
    const images = []
    const videos = []
    const others = []
    for (const f of files) {
      const isDir = f.is_dir === true || f.file_type === 'directory' || f.is_directory === true
      if (isDir) {
        folders.push(f)
      } else if (f.file_type === 1 || ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp', 'tif', 'tiff', 'heic', 'heif', 'avif', 'jxl'].includes(f.extension?.toLowerCase())) {
        images.push(f)
      } else if (f.file_type === 2 || ['mp4', 'mkv', 'webm', 'mov', 'avi', 'flv', 'wmv'].includes(f.extension?.toLowerCase())) {
        videos.push(f)
      } else {
        others.push(f)
      }
    }
    return [
      { title: 'Папки', files: folders },
      { title: 'Изображения', files: images },
      { title: 'Видео', files: videos },
      { title: 'Другие файлы', files: others }
    ].filter(g => g.files.length > 0)
  }

  if (groupBy === 'extension') {
    const groupsMap = {}
    for (const f of files) {
      const isDir = f.is_dir === true || f.file_type === 'directory' || f.is_directory === true
      const ext = isDir ? 'Папка' : (f.extension ? f.extension.toUpperCase() : 'Без расширения')
      if (!groupsMap[ext]) {
        groupsMap[ext] = []
      }
      groupsMap[ext].push(f)
    }
    const keys = Object.keys(groupsMap).sort((a, b) => {
      if (a === 'Папка') return -1
      if (b === 'Папка') return 1
      return a.localeCompare(b)
    })
    return keys.map(k => ({
      title: k === 'Папка' ? 'Папки' : `Файлы ${k}`,
      files: groupsMap[k]
    }))
  }

  if (groupBy === 'size') {
    const huge = []    // > 128 MB
    const large = []   // 1 MB - 128 MB
    const medium = []  // 100 KB - 1 MB
    const small = []   // < 100 KB
    const folders = []
    
    for (const f of files) {
      const isDir = f.is_dir === true || f.file_type === 'directory' || f.is_directory === true
      if (isDir) {
        folders.push(f)
      } else {
        const size = f.size || 0
        if (size >= 128 * 1024 * 1024) {
          huge.push(f)
        } else if (size >= 1 * 1024 * 1024) {
          large.push(f)
        } else if (size >= 100 * 1024) {
          medium.push(f)
        } else {
          small.push(f)
        }
      }
    }
    return [
      { title: 'Папки', files: folders },
      { title: 'Огромные (> 128 МБ)', files: huge },
      { title: 'Крупные (1 МБ - 128 МБ)', files: large },
      { title: 'Средние (100 КБ - 1 МБ)', files: medium },
      { title: 'Маленькие (< 100 КБ)', files: small }
    ].filter(g => g.files.length > 0)
  }

  if (groupBy === 'date') {
    const today = []
    const yesterday = []
    const thisWeek = []
    const thisMonth = []
    const lastMonth = []
    const thisYear = []
    const older = []
    const folders = []

    const now = new Date()
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
    const startOfYesterday = startOfToday - 24 * 60 * 60 * 1000
    const dayOfWeek = now.getDay()
    const daysToMonday = dayOfWeek === 0 ? 6 : dayOfWeek - 1
    const startOfThisWeek = startOfToday - daysToMonday * 24 * 60 * 60 * 1000
    const startOfThisMonth = new Date(now.getFullYear(), now.getMonth(), 1).getTime()
    const startOfLastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1).getTime()
    const startOfThisYear = new Date(now.getFullYear(), 0, 1).getTime()

    for (const f of files) {
      const isDir = f.is_dir === true || f.file_type === 'directory' || f.is_directory === true
      if (isDir) {
        folders.push(f)
      } else {
        const time = f._modifiedTime || 0
        if (time >= startOfToday) {
          today.push(f)
        } else if (time >= startOfYesterday) {
          yesterday.push(f)
        } else if (time >= startOfThisWeek) {
          thisWeek.push(f)
        } else if (time >= startOfThisMonth) {
          thisMonth.push(f)
        } else if (time >= startOfLastMonth) {
          lastMonth.push(f)
        } else if (time >= startOfThisYear) {
          thisYear.push(f)
        } else {
          older.push(f)
        }
      }
    }

    return [
      { title: 'Папки', files: folders },
      { title: 'Сегодня', files: today },
      { title: 'Вчера', files: yesterday },
      { title: 'Ранее на этой неделе', files: thisWeek },
      { title: 'Ранее в этом месяце', files: thisMonth },
      { title: 'В прошлом месяце', files: lastMonth },
      { title: 'Ранее в этом году', files: thisYear },
      { title: 'Давно', files: older }
    ].filter(g => g.files.length > 0)
  }

  return [{ title: 'Все файлы', files }]
}

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

    // Группировка
    groupBy: 'none',       // none, type, date, size, extension

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
    // Initialize with cached count so the badge is correct before first fetch
    trashItems: (() => {
      try {
        const cached = localStorage.getItem('lapai_trash_cache')
        return cached ? JSON.parse(cached) : []
      } catch { return [] }
    })(),
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
        const fromTime = new Date(state.filters.dateFrom).getTime()
        files = files.filter(f => (f._modifiedTime || 0) >= fromTime)
      }
      if (state.filters.dateTo) {
        const toTime = new Date(state.filters.dateTo).getTime()
        files = files.filter(f => (f._modifiedTime || 0) <= toTime)
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
            cmp = (a._modifiedTime || 0) - (b._modifiedTime || 0)
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

      // Группировка
      if (state.groupBy && state.groupBy !== 'none') {
        const groups = groupFilesHelper(files, state.groupBy)
        const flat = []
        for (const g of groups) {
          flat.push(...g.files)
        }
        return flat
      }

      return files
    },

    canUndo: (state) => state.historyIndex >= 0,
    canRedo: (state) => state.historyIndex < state.selectionHistory.length - 1,
  },

  actions: {
    async fetchTrash() {
      try {
        this.trashItems = await invoke('get_trash_contents')
        // Cache in localStorage so badge is instant on next startup
        try {
          localStorage.setItem('lapai_trash_cache', JSON.stringify(this.trashItems))
        } catch {}
      } catch (err) {
        console.error('Failed to load trash:', err)
      }
    },

    setFiles(files) {
      const normalized = files.map(f => ({
        ...f,
        _modifiedTime: f.modified ? new Date(f.modified).getTime() : 0
      }))
      this.files = normalized
      this.filteredFiles = [...normalized]
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
        await this.fetchTrash()
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
        await this.fetchTrash()
      } catch (error) {
        console.error('Failed to undo delete:', error)
        alert('Не удалось отменить удаление: ' + error)
      }
    },

    async copySelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return

      const promises = this.selectedIds.map(async (src) => {
        try {
          const fileName = src.split('\\').pop() || src.split('/').pop()
          await invoke('cross_copy', { src, dest: `${destPath}\\${fileName}` })
        } catch (error) {
          console.error('Failed to copy:', error)
        }
      })
      await Promise.all(promises)
    },

    async moveSelectedFiles(destPath) {
      if (this.selectedIds.length === 0) return

      const promises = this.selectedIds.map(async (src) => {
        try {
          const fileName = src.split('\\').pop() || src.split('/').pop()
          await invoke('cross_move', { src, dest: `${destPath}\\${fileName}` })
        } catch (error) {
          console.error('Failed to move:', error)
        }
      })
      await Promise.all(promises)

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

      const promises = paths.map(async (src) => {
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
      })
      await Promise.all(promises)

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