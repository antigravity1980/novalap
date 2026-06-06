<template>
  <div
    class="virtual-scroll-gallery h-full overflow-y-auto overflow-x-hidden relative"
    ref="containerRef"
    @scroll="onScroll"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseLeave"
    style="user-select: none;"
  >
    <div class="virtual-scroll-spacer" :style="{ height: totalHeight + 'px' }">
      <!-- Visible rows -->
      <div
        v-for="row in visibleRows"
        :key="row.index"
        class="gallery-row flex gap-2 px-2"
        :style="{
          position: 'absolute',
          top: row.top + 'px',
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
          @dblclick.stop="openQuickLook(file)"
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
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue'
import { useGalleryStore } from '../store'
import ThumbnailCard from './ThumbnailCard.vue'

const props = defineProps({
  files: { type: Array, default: () => [] },
  thumbnailSize: { type: Number, default: 200 },
  overscan: { type: Number, default: 3 },
})

const emit = defineEmits(['openQuickLook'])

const galleryStore = useGalleryStore()
const containerRef = ref(null)
const scrollTop = ref(0)
const containerHeight = ref(800)
const containerWidth = ref(1200)
const gap = 8

// Row dimensions
const rowHeight = computed(() => props.thumbnailSize * 0.75 + 40 + gap)
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
  const top = Math.min(dragStart.y, dragCurrent.y) - scrollTop.value
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
  const py = rowIndex * rowHeight.value
  return {
    left: px,
    top: py,
    right: px + props.thumbnailSize,
    bottom: py + (props.thumbnailSize * 0.75 + 40),
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

  e.preventDefault()
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
function onCardClick(e, file) {
  if (e.ctrlKey || e.metaKey) {
    // Toggle single item
    galleryStore.toggleSelection(file.path)
  } else if (e.shiftKey && galleryStore.selectedIds.length > 0) {
    // Range selection from last selected to clicked
    const lastPath = galleryStore.selectedIds[galleryStore.selectedIds.length - 1]
    const lastIndex = props.files.findIndex(f => f.path === lastPath)
    const clickedIndex = props.files.findIndex(f => f.path === file.path)
    if (lastIndex >= 0 && clickedIndex >= 0) {
      const from = Math.min(lastIndex, clickedIndex)
      const to = Math.max(lastIndex, clickedIndex)
      const rangeIds = props.files.slice(from, to + 1).map(f => f.path)
      galleryStore.selectedIds = [...new Set([...galleryStore.selectedIds, ...rangeIds])]
    }
  } else {
    // Single select (clear others)
    galleryStore.selectedIds = [file.path]
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
  border: 1px solid oklch(var(--p));
  background: oklch(var(--p) / 0.15);
  border-radius: 2px;
  z-index: 100;
}
</style>