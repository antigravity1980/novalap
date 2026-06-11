<template>
  <div class="gallery-container w-full h-full relative" @wheel="onWheel">
    <div
      v-if="galleryStore.isLoading"
      class="flex items-center justify-center h-full"
    >
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>

    <div
      v-else-if="galleryStore.displayedFiles.length === 0"
      class="flex flex-col items-center justify-center h-full text-base-content/40 space-y-2"
    >
      <span class="text-4xl">📂</span>
      <span class="text-sm">{{ $t("gallery.no_files") }}</span>
    </div>

    <!-- Virtual scroll container -->
    <div v-else class="w-full h-full">
      <VirtualScrollGallery
        :files="galleryStore.displayedFiles"
        :thumbnailSize="galleryStore.thumbnailSize"
        @openQuickLook="openQuickLook"
      />
    </div>

    <!-- Zoom + spacing controls -->
    <div
      v-if="!galleryStore.focusMode"
      class="zoom-control fixed bottom-8 right-8 flex items-center gap-3 bg-base-300/80 backdrop-blur border border-base-200/50 rounded-lg px-3 py-2 shadow-2xl z-20 hover:border-primary/30 transition-all duration-200"
    >
      <span class="text-xs text-base-content/65">🔍</span>
      <input
        type="range"
        min="0.5"
        max="5.12"
        step="0.1"
        :value="galleryStore.zoomLevel"
        @input="onZoomChange"
        class="range range-xs range-primary w-24"
        title="Размер миниатюр"
      />
      <span class="text-xs font-mono w-10 text-right text-base-content/70"
        >{{ Math.round(galleryStore.thumbnailSize) }}px</span
      >

      <div class="w-px h-5 bg-base-content/15 mx-1"></div>

      <span class="text-xs text-base-content/65">↔</span>
      <input
        type="range"
        min="0"
        max="100"
        step="1"
        :value="Math.round((galleryStore.thumbnailGap / 50) * 100)"
        @input="onGapChange"
        class="range range-xs range-primary w-24"
        title="Промежуток между миниатюрами (0 = вплотную, 100 = 50px)"
      />
      <span class="text-xs font-mono w-10 text-right text-base-content/70"
        >{{ galleryStore.thumbnailGap }}px</span
      >
    </div>
  </div>
</template>

<script setup>
import { useGalleryStore } from "../store";
import VirtualScrollGallery from "./VirtualScrollGallery.vue";

const galleryStore = useGalleryStore();

const emit = defineEmits(["openQuickLook"]);

function onZoomChange(event) {
  galleryStore.setZoom(parseFloat(event.target.value));
}

function onGapChange(event) {
  // 0..100% → 0..50 px (линейно). 0 = миниатюры вплотную, 100 = 50px зазор.
  const percent = parseFloat(event.target.value);
  galleryStore.setThumbnailGap((percent / 100) * 50);
}

function onWheel(event) {
  if (event.ctrlKey || event.metaKey) {
    event.preventDefault();
    const delta = event.deltaY > 0 ? -0.1 : 0.1;
    galleryStore.setZoom(galleryStore.zoomLevel + delta);
  }
}

function openQuickLook(file) {
  emit("openQuickLook", file);
}
</script>

<style scoped>
.zoom-control {
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
}
</style>
