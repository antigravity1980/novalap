<template>
  <div
    class="thumbnail-card rounded-xl overflow-hidden border cursor-pointer transition-all duration-200 bg-base-200/40 relative flex flex-col justify-between"
    :class="{
      'border-primary ring-2 ring-primary/45 shadow-lg shadow-primary/10 translate-y-[-2px] bg-base-100': selected,
      'border-base-content/5 hover:border-primary/20 hover:shadow-xl hover:translate-y-[-2px] hover:bg-base-100/30': !selected,
    }"
    :style="{ width: size + 'px' }"
    @click="$emit('click')"
    @dblclick="$emit('dblclick')"
  >
    <!-- Thumbnail Image Container -->
    <div
      class="thumbnail-image bg-base-300 flex items-center justify-center overflow-hidden relative select-none w-full"
      :style="{ height: size * 0.75 + 'px' }"
    >
      <img
        v-if="isImage"
        :src="getThumbnailUrl(file.path)"
        :alt="file.name"
        class="w-full h-full object-cover transition-transform duration-300 hover:scale-105"
        loading="lazy"
      />
      <!-- Video tag/icon overlay -->
      <div v-else-if="isVideo" class="w-full h-full flex flex-col items-center justify-center gap-1.5 bg-gradient-to-br from-base-300 to-base-200 text-base-content/40 hover:text-base-content/60">
        <span class="text-3xl filter drop-shadow">🎬</span>
        <span class="text-[10px] uppercase font-bold tracking-wider opacity-60">{{ $t('gallery.video_label') }}</span>
        <div class="absolute bottom-2 right-2 bg-black/60 backdrop-blur rounded px-1.5 py-0.5 text-[10px] text-white font-mono flex items-center gap-1">
          <span>▶</span>
          <span>{{ $t('gallery.video_label').toUpperCase() }}</span>
        </div>
      </div>
      <!-- Папка -->
      <div v-else-if="isFolder" class="w-full h-full flex flex-col items-center justify-center gap-1.5 bg-base-300/30 text-base-content/50">
        <span class="text-4xl">📁</span>
        <span class="text-[10px] uppercase font-bold tracking-wider">{{ $t('gallery.folder_label').toUpperCase() }}</span>
      </div>
      <!-- Other files generic -->
      <div v-else class="w-full h-full flex flex-col items-center justify-center gap-1.5 bg-base-300/30 text-base-content/40">
        <span class="text-3xl">📄</span>
        <span class="text-[10px] uppercase font-bold tracking-wider">{{ file.extension?.toUpperCase() || $t('gallery.file_label').toUpperCase() }}</span>
      </div>

      <!-- AI Source indicator (corner badge) -->
      <div v-if="file.ai_source" class="absolute top-2 left-2 z-10">
        <span
          class="badge badge-xs text-[9px] font-bold py-1 px-1.5 border border-white/10 shadow shadow-black/20"
          :class="getAiSourceClass(file.ai_source)"
        >
          {{ file.ai_source }}
        </span>
      </div>

      <!-- Selection checkmark badge -->
      <div v-if="selected" class="absolute top-2 right-2 bg-primary text-primary-content w-4 h-4 rounded-full flex items-center justify-center text-[9px] font-bold border border-white/20 shadow-md">
        ✓
      </div>
    </div>

    <!-- Info row -->
    <div class="thumbnail-info p-2.5 text-xs flex flex-col gap-1 shrink-0 border-t border-base-content/5 bg-base-200/20">
      <div class="file-name truncate font-medium text-base-content/90" :title="file.name">
        {{ file.name }}
      </div>
      <div class="flex items-center justify-between mt-0.5 text-[10px] text-base-content/40 font-mono">
        <span v-if="file.resolution" class="font-semibold">
          {{ file.resolution.width }}×{{ file.resolution.height }}
        </span>
        <span v-else>—</span>
        <span>{{ formatBytes(file.size) }}</span>
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

const isFolder = computed(() => {
  return props.file.is_dir === true || props.file.file_type === 'directory' || props.file.is_directory === true
})

const isImage = computed(() => {
  const ext = props.file.extension?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'avif', 'jxl', 'svg', 'ico'].includes(ext)
})

const isVideo = computed(() => {
  const ext = props.file.extension?.toLowerCase()
  return ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mpeg', '3gp'].includes(ext)
})

function getThumbnailUrl(filePath) {
  return `asset://localhost/${encodeURI(filePath)}`
}

function formatBytes(bytes) {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(0)} ${units[unitIndex]}`
}

function getAiSourceClass(source) {
  const s = source.toLowerCase()
  if (s.includes('comfyui')) {
    return 'bg-purple-600/90 text-white'
  }
  if (s.includes('midjourney')) {
    return 'bg-blue-600/90 text-white'
  }
  if (s.includes('stable diffusion') || s.includes('sd_')) {
    return 'bg-teal-600/90 text-white'
  }
  if (s.includes('gpt')) {
    return 'bg-emerald-600/90 text-white'
  }
  if (s.includes('grok')) {
    return 'bg-amber-600/90 text-white'
  }
  if (s.includes('nano banana')) {
    return 'bg-rose-600/90 text-white'
  }
  return 'bg-neutral-600/90 text-white'
}
</script>

<style scoped>
.thumbnail-card {
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), box-shadow 0.2s, border-color 0.2s, background-color 0.2s;
}
.thumbnail-image img {
  pointer-events: none;
  backface-visibility: hidden;
}
</style>