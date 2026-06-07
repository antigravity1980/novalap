<template>
  <div
    class="virtual-scroll-gallery h-full overflow-y-auto overflow-x-hidden relative focus:outline-none"
    ref="containerRef"
    tabindex="0"
    @scroll="onScroll"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseLeave"
    @keydown="onKeyDown"
    @contextmenu.prevent.stop="handleContextMenu"
    style="user-select: none;"
  >
    <div class="virtual-scroll-spacer" :style="{ height: (totalHeight + topPadding * 2) + 'px' }">
      <!-- Visible rows -->
      <div
        v-for="row in visibleRows"
        :key="row.index"
        class="gallery-row flex gap-2 px-2"
        :style="{
          position: 'absolute',
          top: (row.top + topPadding) + 'px',
          left: 0,
          right: 0,
          height: rowHeight + 'px',
        }"
      >
        <ThumbnailCard
          v-for="file in row.files"
          :key="file.path"
          :file="file"
          :size="thumbnailSize"
          :selected="galleryStore.selectedIds.includes(file.path)"
          @click.stop="onCardClick($event, file)"
          @dblclick.stop="onCardDblClick(file)"
          class="flex-shrink-0"
          :style="{ width: thumbnailSize + 'px' }"
          :data-path="file.path"
        />
      </div>
    </div>

    <!-- Rubber-band selection rectangle -->
    <div
      v-if="isDragging"
      class="rubber-band"
      :style="rubberBandStyle"
    />

    <!-- Context menu on empty space -->
    <ContextMenu
      ref="contextMenuRef"
      :menuItems="contextMenuItems"
      :smallIcon="true"
      style="display: none;"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue'
import { useGalleryStore } from '../store'
import { useNavigationStore } from '../../navigation/store'
import { invoke } from '@tauri-apps/api/core'
import ThumbnailCard from './ThumbnailCard.vue'
import ContextMenu from '@/components/ContextMenu.vue'

const props = defineProps({
  files: { type: Array, default: () => [] },
  thumbnailSize: { type: Number, default: 200 },
  overscan: { type: Number, default: 3 },
})

const emit = defineEmits(['openQuickLook'])

const galleryStore = useGalleryStore()
const navigationStore = useNavigationStore()
const containerRef = ref(null)
const contextMenuRef = ref(null)
const scrollTop = ref(0)
const containerHeight = ref(800)
const containerWidth = ref(1200)
const gap = 8
const topPadding = 16

// Row dimensions
const rowHeight = computed(() => props.thumbnailSize * 0.75 + 64 + gap)
const colsPerRow = computed(() => {
  const width = containerWidth.value || 1200
  return Math.max(1, Math.floor((width + gap) / (props.thumbnailSize + gap)))
})

// Group files into rows
const rows = computed(() => {
  const cols = colsPerRow.value
  const result = []
  for (let i = 0; i < props.files.length; i += cols) {
    result.push({
      index: result.length,
      files: props.files.slice(i, i + cols),
    })
  }
  return result
})

const totalHeight = computed(() => rows.value.length * rowHeight.value)

// Visible rows based on scroll position
const visibleRows = computed(() => {
  const startRow = Math.max(0, Math.floor(scrollTop.value / rowHeight.value) - props.overscan)
  const endRow = Math.min(
    rows.value.length,
    Math.ceil((scrollTop.value + containerHeight.value) / rowHeight.value) + props.overscan
  )
  return rows.value.slice(startRow, endRow).map(row => ({
    ...row,
    top: row.index * rowHeight.value,
  }))
})

// ─── Rubber-band selection ───────────────────────────────────────────────────
const isDragging = ref(false)
const dragStart = reactive({ x: 0, y: 0 }) // relative to container (incl. scroll)
const dragCurrent = reactive({ x: 0, y: 0 })
let selectionBeforeDrag = []

const rubberBandStyle = computed(() => {
  const left = Math.min(dragStart.x, dragCurrent.x)
  const top = Math.min(dragStart.y, dragCurrent.y)
  const width = Math.abs(dragCurrent.x - dragStart.x)
  const height = Math.abs(dragCurrent.y - dragStart.y)
  return {
    left: left + 'px',
    top: top + 'px',
    width: width + 'px',
    height: height + 'px',
  }
})

function getCardRect(fileIndex) {
  // Calculate the card bounds in document-scroll space
  const cols = colsPerRow.value
  const rowIndex = Math.floor(fileIndex / cols)
  const colIndex = fileIndex % cols
  const px = 8 + colIndex * (props.thumbnailSize + gap) // px-2 = 8px padding
  const py = rowIndex * rowHeight.value + topPadding
  return {
    left: px,
    top: py,
    right: px + props.thumbnailSize,
    bottom: py + (props.thumbnailSize * 0.75 + 64),
  }
}

function getFilesInRect(rx1, ry1, rx2, ry2) {
  const left = Math.min(rx1, rx2)
  const right = Math.max(rx1, rx2)
  const top = Math.min(ry1, ry2)
  const bottom = Math.max(ry1, ry2)

  return props.files.filter((_, i) => {
    const r = getCardRect(i)
    return r.right > left && r.left < right && r.bottom > top && r.top < bottom
  }).map(f => f.path)
}

function onMouseDown(e) {
  // Only start rubber-band on the background (not on a card)
  if (e.button !== 0) return
  const target = e.target
  if (target.closest('.thumbnail-card')) return

  const rect = containerRef.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top + scrollTop.value

  isDragging.value = true
  dragStart.x = x
  dragStart.y = y
  dragCurrent.x = x
  dragCurrent.y = y

  // Save current selection for Shift/Ctrl combination (extend)
  if (!e.ctrlKey && !e.shiftKey) {
    galleryStore.clearSelection()
    selectionBeforeDrag = []
  } else {
    selectionBeforeDrag = [...galleryStore.selectedIds]
  }
}

function onMouseMove(e) {
  if (!isDragging.value) return
  const rect = containerRef.value.getBoundingClientRect()
  dragCurrent.x = e.clientX - rect.left
  dragCurrent.y = e.clientY - rect.top + scrollTop.value

  // Update selection
  const inRect = getFilesInRect(dragStart.x, dragStart.y, dragCurrent.x, dragCurrent.y)
  const merged = [...new Set([...selectionBeforeDrag, ...inRect])]
  galleryStore.selectedIds = merged
}

function onMouseUp(e) {
  if (isDragging.value) {
    isDragging.value = false
  }
}

function onMouseLeave(e) {
  if (isDragging.value) {
    isDragging.value = false
  }
}

// ─── Card click with Ctrl/Shift support ─────────────────────────────────────
const anchorIndex = ref(-1)

function onCardClick(e, file) {
  const clickedIndex = props.files.findIndex(f => f.path === file.path)
  containerRef.value?.focus()

  if (galleryStore.selectionMode || e.ctrlKey || e.metaKey) {
    // Toggle single item
    galleryStore.toggleSelection(file.path)
    anchorIndex.value = clickedIndex
  } else if (e.shiftKey && galleryStore.selectedIds.length > 0) {
    // Range selection from anchor to clicked
    if (anchorIndex.value === -1) {
      const lastPath = galleryStore.selectedIds[0]
      anchorIndex.value = props.files.findIndex(f => f.path === lastPath)
    }
    if (anchorIndex.value >= 0 && clickedIndex >= 0) {
      const from = Math.min(anchorIndex.value, clickedIndex)
      const to = Math.max(anchorIndex.value, clickedIndex)
      const rangeIds = props.files.slice(from, to + 1).map(f => f.path)
      galleryStore.selectedIds = rangeIds
    }
  } else {
    // Single select (clear others)
    galleryStore.selectedIds = [file.path]
    anchorIndex.value = clickedIndex
  }
}

async function onCardDblClick(file) {
  const isFolder = file.is_dir === true || file.file_type === 'directory' || file.is_directory === true
  if (isFolder) {
    await navigationStore.navigateTo(file.path)
    galleryStore.setFiles(navigationStore.folders)
  } else {
    emit('openQuickLook', file)
  }
}

// ─── Context Menu for empty space ──────────────────────────────────────────
const contextMenuItems = computed(() => {
  const items = []
  if (galleryStore.clipboard.paths.length > 0) {
    items.push({
      label: 'Вставить',
      action: () => galleryStore.paste(navigationStore.currentPath)
    })
  }
  items.push({
    label: 'Создать папку',
    action: () => createFolderInCurrentDir()
  })
  items.push({
    label: 'Выделить всё',
    action: () => galleryStore.selectAll()
  })
  return items
})

function handleContextMenu(e) {
  // Only open empty space menu if not clicking inside a card
  if (e.target.closest('.thumbnail-card')) return
  contextMenuRef.value?.open(e.clientX, e.clientY)
}

async function createFolderInCurrentDir() {
  const name = prompt('Введите имя папки:')
  if (name && navigationStore.currentPath) {
    const separator = navigationStore.currentPath.includes('/') ? '/' : '\\'
    const newPath = navigationStore.currentPath.endsWith(separator)
      ? navigationStore.currentPath + name
      : navigationStore.currentPath + separator + name
    try {
      await invoke('mkdir_folder', { path: newPath })
      await navigationStore.refresh()
      galleryStore.setFiles(navigationStore.folders)
    } catch (e) {
      alert('Не удалось создать папку: ' + e)
    }
  }
}

// ─── Keyboard Navigation ─────────────────────────────────────────────────────
function scrollToIndex(index) {
  const cols = colsPerRow.value
  const rowIndex = Math.floor(index / cols)
  const rowTop = rowIndex * rowHeight.value + topPadding
  const rowBottom = rowTop + rowHeight.value

  const container = containerRef.value
  if (!container) return

  const curScrollTop = container.scrollTop
  const viewHeight = containerHeight.value

  if (rowTop < curScrollTop) {
    container.scrollTop = rowTop
  } else if (rowBottom > curScrollTop + viewHeight) {
    container.scrollTop = rowBottom - viewHeight
  }
}

function onKeyDown(e) {
  // Ctrl+A Select All
  if (e.ctrlKey && (e.code === 'KeyA' || e.key.toLowerCase() === 'a' || e.key === 'ф' || e.key === 'Ф')) {
    e.preventDefault()
    galleryStore.selectAll()
    return
  }

  if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(e.key)) {
    e.preventDefault()
    if (props.files.length === 0) return

    let currentIndex = -1
    if (galleryStore.selectedIds.length > 0) {
      const lastSelectedPath = galleryStore.selectedIds[galleryStore.selectedIds.length - 1]
      currentIndex = props.files.findIndex(f => f.path === lastSelectedPath)
    }

    if (currentIndex === -1) {
      currentIndex = 0
    }

    const cols = colsPerRow.value
    let nextIndex = currentIndex

    if (e.key === 'ArrowLeft') {
      nextIndex = Math.max(0, currentIndex - 1)
    } else if (e.key === 'ArrowRight') {
      nextIndex = Math.min(props.files.length - 1, currentIndex + 1)
    } else if (e.key === 'ArrowUp') {
      nextIndex = Math.max(0, currentIndex - cols)
    } else if (e.key === 'ArrowDown') {
      nextIndex = Math.min(props.files.length - 1, currentIndex + cols)
    }

    if (e.shiftKey) {
      if (anchorIndex.value === -1) {
        anchorIndex.value = currentIndex
      }
      const from = Math.min(anchorIndex.value, nextIndex)
      const to = Math.max(anchorIndex.value, nextIndex)
      const rangeIds = props.files.slice(from, to + 1).map(f => f.path)
      galleryStore.selectedIds = rangeIds
    } else {
      anchorIndex.value = nextIndex
      galleryStore.selectedIds = [props.files[nextIndex].path]
    }

    scrollToIndex(nextIndex)
  } else if (e.key === ' ') {
    e.preventDefault()
    if (galleryStore.selectedIds.length > 0) {
      const lastSelectedPath = galleryStore.selectedIds[galleryStore.selectedIds.length - 1]
      const file = props.files.find(f => f.path === lastSelectedPath)
      if (file) {
        onCardDblClick(file)
      }
    }
  }
}

// ─── Resize observer ─────────────────────────────────────────────────────────
let resizeObserver = null

onMounted(() => {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight || 800
    containerWidth.value = containerRef.value.clientWidth || 1200
    resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height
        containerWidth.value = entry.contentRect.width
      }
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) resizeObserver.disconnect()
})

function onScroll() {
  scrollTop.value = containerRef.value?.scrollTop || 0
}

function openQuickLook(file) {
  emit('openQuickLook', file)
}
</script>

<style scoped>
.virtual-scroll-gallery {
  position: relative;
  overflow-anchor: none;
  overflow-x: hidden;
  cursor: default;
}
.virtual-scroll-spacer {
  position: relative;
}
.gallery-row {
  display: flex;
  align-items: flex-start;
}
/* Rubber-band selection rectangle */
.rubber-band {
  position: absolute;
  pointer-events: none;
  border: 1.5px solid rgba(59, 130, 246, 0.8) !important;
  background-color: rgba(59, 130, 246, 0.15) !important;
  border-radius: 2px;
  z-index: 99999 !important;
}
</style>