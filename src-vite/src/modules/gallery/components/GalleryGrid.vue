<template>
  <div class="gallery-container w-full h-full overflow-auto" @wheel.prevent="onWheel">
    <div v-if="galleryStore.isLoading" class="flex items-center justify-center h-32">
      <span class="loading loading-spinner loading-md text-primary"></span>
    </div>

    <div v-else-if="galleryStore.displayedFiles.length === 0" class="flex items-center justify-center h-32 text-base-content/50">
      No files found
    </div>

    <div
      v-else
      class="gallery-grid"
      :style="{
        gridTemplateColumns: `repeat(auto-fill, minmax(${galleryStore.thumbnailSize}px, 1fr))`,
        gap: `${gridGap}px`,
        padding: `${gridGap}px`,
      }"
    >
      <ThumbnailCard
        v-for="file in galleryStore.displayedFiles"
        :key="file.path"
        :file="file"
        :size="galleryStore.thumbnailSize"
        :selected="galleryStore.selectedIds.includes(file.path)"
        @click="galleryStore.toggleSelection(file.path)"
        @dblclick="openQuickLook(file)"
      />
    </div>

    <!-- Zoom slider -->
    <div class="zoom-control fixed bottom-4 right-4 flex items-center gap-2 bg-base-200/80 backdrop-blur rounded-lg px-3 py-2 shadow-lg">
      <span class="text-xs">🔍</span>
      <input
        type="range"
        min="0.5"
        max="3"
        step="0.1"
        :value="galleryStore.zoomLevel"
        @input="onZoomChange"
        class="range range-xs w-24"
      />
      <span class="text-xs w-8 text-right">{{ Math.round(galleryStore.thumbnailSize) }}px</span>
    </div>
  </div>
</template>

<script setup>
import { useGalleryStore } from '../store'
import ThumbnailCard from './ThumbnailCard.vue'

const galleryStore = useGalleryStore()
const gridGap = 8

const emit = defineEmits(['openQuickLook'])

function onZoomChange(event) {
  galleryStore.setZoom(parseFloat(event.target.value))
}

function onWheel(event) {
  if (event.ctrlKey || event.metaKey) {
    const delta = event.deltaY > 0 ? -0.1 : 0.1
    galleryStore.setZoom(galleryStore.zoomLevel + delta)
    event.preventDefault()
  }
}

function openQuickLook(file) {
  emit('openQuickLook', file)
}
</script>

<style scoped>
.gallery-container {
  scroll-behavior: smooth;
}
.gallery-grid {
  display: grid;
  align-content: start;
}
</style>