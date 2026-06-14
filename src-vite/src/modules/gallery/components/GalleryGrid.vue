<template>
  <div class="gallery-container w-full h-full relative" @wheel="onWheel">
    <div
      v-if="navigationStore.isLoading"
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

    <!-- Floating controls container (Zoom, Spacing & Progress Bar) -->
    <div
      v-if="!galleryStore.focusMode"
      class="fixed bottom-8 right-8 flex flex-col items-end gap-2.5 z-20 select-none pointer-events-none"
    >
      <!-- Paste Progress Bar -->
      <div
        v-if="galleryStore.pasteProgress.show"
        class="pointer-events-auto flex flex-col gap-2 p-3.5 rounded-lg bg-base-300/90 backdrop-blur-md border border-neutral/30 shadow-2xl w-80 text-xs transition-all duration-300"
      >
        <div class="flex justify-between items-center font-bold text-base-content/90">
          <span class="flex items-center gap-1.5">
            <svg class="animate-spin h-3.5 w-3.5 text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Копирование файлов...
          </span>
          <span class="font-mono text-base-content/60">{{ galleryStore.pasteProgress.current }} / {{ galleryStore.pasteProgress.total }}</span>
        </div>
        <div class="w-full bg-base-100/50 rounded-full h-2 overflow-hidden border border-neutral/15">
          <div
            class="bg-primary h-full rounded-full transition-all duration-300 ease-out"
            :style="{ width: galleryStore.pasteProgress.percentage + '%' }"
          ></div>
        </div>
        <div class="flex justify-between text-[10px] text-base-content/40 font-medium">
          <span>Готово: {{ galleryStore.pasteProgress.percentage }}%</span>
          <span>Вставка...</span>
        </div>
      </div>

      <!-- Zoom + spacing controls -->
      <div
        class="zoom-control pointer-events-auto flex items-center gap-3 bg-base-300/80 backdrop-blur border border-base-200/50 rounded-lg px-3 py-2 shadow-2xl hover:border-primary/30 transition-all duration-200"
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
  </div>
</template>

<script setup>
import { useGalleryStore } from "../store";
import { useNavigationStore } from "../../navigation/store";
import VirtualScrollGallery from "./VirtualScrollGallery.vue";

const galleryStore = useGalleryStore();
const navigationStore = useNavigationStore();

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
