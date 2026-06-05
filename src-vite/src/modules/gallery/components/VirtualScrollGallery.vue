<template>
  <div class="virtual-scroll-gallery h-full overflow-auto" ref="containerRef" @scroll="onScroll">
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
          @click="galleryStore.toggleSelection(file.path)"
          @dblclick="openQuickLook(file)"
          class="flex-shrink-0"
          :style="{ width: thumbnailSize + 'px' }"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useGalleryStore } from '../store'
import ThumbnailCard from './ThumbnailCard.vue'

const props = defineProps({
  files: { type: Array, default: () => [] },
  thumbnailSize: { type: Number, default: 200 },
  overscan: { type: Number, default: 3 }, // extra rows to render
})

const emit = defineEmits(['openQuickLook'])

const galleryStore = useGalleryStore()
const containerRef = ref(null)
const scrollTop = ref(0)
const containerHeight = ref(800)
const gap = 8

// Row dimensions
const rowHeight = computed(() => props.thumbnailSize * 0.75 + 40 + gap) // image height + info + gap
const colsPerRow = computed(() => {
  if (!containerRef.value) return 4
  const containerWidth = containerRef.value.clientWidth || 1200
  return Math.max(1, Math.floor((containerWidth + gap) / (props.thumbnailSize + gap)))
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

// Resize observer
let resizeObserver = null

onMounted(() => {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight || 800
    resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height
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
}
.virtual-scroll-spacer {
  position: relative;
}
.gallery-row {
  display: flex;
  align-items: flex-start;
}
</style>