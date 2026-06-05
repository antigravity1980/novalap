<template>
  <teleport to="body">
    <div
      v-if="visible"
      class="quick-look-overlay fixed inset-0 z-50 bg-black/90 flex items-center justify-center"
      @click.self="close"
      @keydown.escape="close"
      @keydown.space.prevent="close"
      @keydown.left.prevent="prev"
      @keydown.right.prevent="next"
      tabindex="0"
      ref="overlayRef"
    >
      <!-- Close button -->
      <button class="absolute top-4 right-4 text-white/60 hover:text-white text-2xl z-10" @click="close">
        ✕
      </button>

      <!-- File counter -->
      <div class="absolute top-4 left-4 text-white/60 text-sm">
        {{ currentIndex + 1 }} / {{ files.length }}
      </div>

      <!-- Navigation arrows -->
      <button
        v-if="currentIndex > 0"
        class="absolute left-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white text-4xl z-10"
        @click="prev"
      >
        ‹
      </button>
      <button
        v-if="currentIndex < files.length - 1"
        class="absolute right-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white text-4xl z-10"
        @click="next"
      >
        ›
      </button>

      <!-- Image preview -->
      <div class="preview-content max-w-[90vw] max-h-[90vh] flex flex-col items-center">
        <img
          v-if="isCurrentImage"
          :src="currentFileUrl"
          class="max-w-full max-h-[85vh] object-contain rounded"
          @error="onImageError"
        />
        <video
          v-else-if="isCurrentVideo"
          :src="currentFileUrl"
          class="max-w-full max-h-[85vh] rounded"
          controls
          autoplay
        ></video>
        <div v-else class="text-white/50 text-lg">
          Unsupported file type: {{ currentFile?.extension }}
        </div>

        <!-- File info bar -->
        <div class="file-info-bar mt-2 text-white/50 text-sm flex items-center gap-4">
          <span>{{ currentFile?.name }}</span>
          <span v-if="currentFile?.resolution">
            {{ currentFile.resolution.width }}x{{ currentFile.resolution.height }}
          </span>
          <span>{{ formatFileSize(currentFile?.size) }}</span>
          <span v-if="currentFile?.ai_source" class="text-primary/70">
            {{ currentFile.ai_source }}
          </span>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  visible: { type: Boolean, default: false },
  files: { type: Array, default: () => [] },
  initialIndex: { type: Number, default: 0 },
})

const emit = defineEmits(['close', 'update:visible'])

const overlayRef = ref(null)
const currentIndex = ref(0)

const currentFile = computed(() => props.files[currentIndex.value] || null)
const currentFileUrl = computed(() => {
  if (!currentFile.value) return ''
  return `asset://localhost/${encodeURI(currentFile.value.path)}`
})

const isCurrentImage = computed(() => {
  const ext = currentFile.value?.extension?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'avif', 'jxl', 'svg', 'ico'].includes(ext)
})

const isCurrentVideo = computed(() => {
  const ext = currentFile.value?.extension?.toLowerCase()
  return ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mpeg', '3gp'].includes(ext)
})

watch(() => props.visible, (val) => {
  if (val) {
    currentIndex.value = props.initialIndex
    nextTick(() => overlayRef.value?.focus())
    document.addEventListener('keydown', handleKeyDown)
  } else {
    document.removeEventListener('keydown', handleKeyDown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})

function handleKeyDown(event) {
  if (!props.visible) return

  switch (event.key) {
    case 'Escape':
    case ' ':
      close()
      break
    case 'ArrowLeft':
      prev()
      break
    case 'ArrowRight':
      next()
      break
  }
}

function close() {
  emit('update:visible', false)
  emit('close')
}

function next() {
  if (currentIndex.value < props.files.length - 1) {
    currentIndex.value++
  }
}

function prev() {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

function formatFileSize(bytes) {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`
}

function onImageError(event) {
  event.target.style.display = 'none'
  // Показываем fallback
}
</script>

<style scoped>
.quick-look-overlay {
  backdrop-filter: blur(4px);
}
.quick-look-overlay:focus {
  outline: none;
}
</style>