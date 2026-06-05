<template>
  <div
    class="thumbnail-card rounded-lg overflow-hidden border cursor-pointer transition-all duration-150"
    :class="{
      'border-primary ring-2 ring-primary': selected,
      'border-base-200 hover:border-base-300 hover:shadow-md': !selected,
    }"
    :style="{ maxWidth: size + 'px' }"
    @click="$emit('click')"
    @dblclick="$emit('dblclick')"
  >
    <!-- Thumbnail -->
    <div
      class="thumbnail-image bg-base-200 flex items-center justify-center overflow-hidden"
      :style="{ height: size * 0.75 + 'px' }"
    >
      <img
        v-if="isImage"
        :src="getThumbnailUrl(file.path)"
        :alt="file.name"
        class="w-full h-full object-cover"
        loading="lazy"
      />
      <div v-else-if="isVideo" class="flex flex-col items-center gap-1 text-base-content/50">
        <span class="text-3xl">🎬</span>
        <span class="text-xs">Video</span>
      </div>
      <div v-else class="flex flex-col items-center gap-1 text-base-content/50">
        <span class="text-3xl">📄</span>
        <span class="text-xs">{{ file.extension?.toUpperCase() }}</span>
      </div>
    </div>

    <!-- Info row -->
    <div class="thumbnail-info p-1.5 text-xs">
      <div class="file-name truncate font-medium" :title="file.name">
        {{ file.name }}
      </div>
      <div v-if="file.resolution" class="resolution text-base-content/60">
        {{ file.resolution.width }}x{{ file.resolution.height }}
      </div>
      <div v-if="file.ai_source" class="ai-source text-primary/70 text-[10px]">
        {{ file.ai_source }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  file: { type: Object, required: true },
  size: { type: Number, default: 200 },
  selected: { type: Boolean, default: false },
})

defineEmits(['click', 'dblclick'])

const isImage = computed(() => {
  const ext = props.file.extension?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'avif', 'jxl'].includes(ext)
})

const isVideo = computed(() => {
  const ext = props.file.extension?.toLowerCase()
  return ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv'].includes(ext)
})

function getThumbnailUrl(filePath) {
  // Используем Tauri asset protocol для загрузки миниатюры
  // В будущем можно добавить генерацию миниатюр через Rust
  return `asset://localhost/${encodeURI(filePath)}`
}
</script>

<style scoped>
.thumbnail-card {
  background: var(--fallback-b1, oklch(var(--b1)));
  transition: transform 0.1s, box-shadow 0.1s;
}
.thumbnail-card:hover {
  transform: scale(1.02);
}
.thumbnail-image {
  position: relative;
}
.thumbnail-image img {
  pointer-events: none;
}
</style>