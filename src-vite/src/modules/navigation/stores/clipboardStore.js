import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

/**
 * Буфер обмена для операций Копировать/Вырезать/Вставить
 */
export const useClipboardStore = defineStore('clipboard', {
  state: () => ({
    /** Массив путей, которые скопированы или вырезаны */
    items: [],
    /** Режим: 'copy' или 'cut' */
    mode: null,
  }),

  getters: {
    hasItems: (state) => state.items.length > 0,
    isCut: (state) => state.mode === 'cut',
    isCopy: (state) => state.mode === 'copy',
  },

  actions: {
    /** Скопировать пути в буфер */
    copy(paths) {
      const arr = Array.isArray(paths) ? paths : [paths]
      this.items = [...arr]
      this.mode = 'copy'
    },

    /** Вырезать пути в буфер */
    cut(paths) {
      const arr = Array.isArray(paths) ? paths : [paths]
      this.items = [...arr]
      this.mode = 'cut'
    },

    /** Вставить элементы из буфера в папку destPath */
    async paste(destPath) {
      if (!this.items.length || !this.mode) return

      const normalizedDest = destPath.endsWith('\\') || destPath.endsWith('/') 
        ? destPath 
        : destPath + '\\'

      try {
        for (const src of this.items) {
          const lastSlash = Math.max(src.lastIndexOf('\\'), src.lastIndexOf('/'))
          const fileName = lastSlash !== -1 ? src.substring(lastSlash + 1) : src
          const dest = normalizedDest + fileName

          if (src.toLowerCase() === dest.toLowerCase()) continue

          if (this.mode === 'cut') {
            await invoke('cross_move', { src, dest })
          } else {
            await invoke('cross_copy', { src, dest })
          }
        }
      } finally {
        // После вставки в режиме вырезания — очищаем буфер
        if (this.mode === 'cut') {
          this.clear()
        }
      }
    },

    /** Очистить буфер */
    clear() {
      this.items = []
      this.mode = null
    },
  },
})