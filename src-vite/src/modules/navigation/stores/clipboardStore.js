import { defineStore } from 'pinia'
import { useGalleryStore } from '../../gallery/store'

/**
 * Буфер обмена для операций Копировать/Вырезать/Вставить
 * Синхронизирован с useGalleryStore.clipboard
 */
export const useClipboardStore = defineStore('clipboard', {
  state: () => ({
    // Состояние делегируется в galleryStore
  }),

  getters: {
    items() {
      const galleryStore = useGalleryStore()
      return galleryStore.clipboard.paths
    },
    mode() {
      const galleryStore = useGalleryStore()
      return galleryStore.clipboard.action
    },
    hasItems() {
      const galleryStore = useGalleryStore()
      return galleryStore.clipboard.paths.length > 0
    },
    isCut() {
      const galleryStore = useGalleryStore()
      return galleryStore.clipboard.action === 'cut'
    },
    isCopy() {
      const galleryStore = useGalleryStore()
      return galleryStore.clipboard.action === 'copy'
    },
  },

  actions: {
    /** Скопировать пути в буфер */
    copy(paths) {
      const galleryStore = useGalleryStore()
      const arr = Array.isArray(paths) ? paths : [paths]
      galleryStore.setClipboard('copy', arr)
    },

    /** Вырезать пути в буфер */
    cut(paths) {
      const galleryStore = useGalleryStore()
      const arr = Array.isArray(paths) ? paths : [paths]
      galleryStore.setClipboard('cut', arr)
    },

    /** Вставить элементы из буфера в папку destPath */
    async paste(destPath) {
      const galleryStore = useGalleryStore()
      await galleryStore.paste(destPath)
    },

    /** Очистить буфер */
    clear() {
      const galleryStore = useGalleryStore()
      galleryStore.clearClipboard()
    },
  },
})