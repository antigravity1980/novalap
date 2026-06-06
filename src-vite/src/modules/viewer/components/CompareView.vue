<template>
  <div class="compare-view fixed inset-0 z-40 bg-black/95 flex flex-col" @keydown.escape="close">
    <!-- Top bar -->
    <div class="compare-header flex items-center justify-between px-4 py-2 bg-base-200/20">
      <h3 class="text-white/80 text-sm">{{ $t('viewer.compare_title', { left: leftFile?.name || '', right: rightFile?.name || '' }) }}</h3>
      <button class="text-white/60 hover:text-white text-xl" @click="close">✕</button>
    </div>

    <!-- Compare modes -->
    <div class="compare-controls flex items-center justify-center gap-4 px-4 py-1 bg-base-200/10">
      <button
        v-for="mode in modes"
        :key="mode.id"
        class="btn btn-ghost btn-xs text-white/60"
        :class="{ 'text-primary bg-primary/10': compareMode === mode.id }"
        @click="compareMode = mode.id"
      >
        {{ $t('viewer.compare_modes.' + mode.id) }}
      </button>
    </div>

    <!-- Comparison area -->
    <div class="compare-area flex-1 relative overflow-hidden">
      <!-- Side-by-side mode -->
      <div v-if="compareMode === 'sidebyside'" class="flex h-full">
        <div class="flex-1 flex items-center justify-center border-r border-white/10">
          <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="max-w-full max-h-full object-contain" />
        </div>
        <div class="flex-1 flex items-center justify-center">
          <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="max-w-full max-h-full object-contain" />
        </div>
      </div>

      <!-- Slider (curtain) mode -->
      <div v-else-if="compareMode === 'slider'" class="h-full relative" ref="sliderContainer"
        @mousemove="onSliderMove" @touchmove="onSliderMove">
        <div class="absolute inset-0">
          <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="w-full h-full object-contain" />
        </div>
        <div class="absolute inset-0 overflow-hidden" :style="{ width: sliderPosition + '%' }">
          <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="w-full h-full object-contain"
            :style="{ width: (100 / sliderPosition * 100) + '%', maxWidth: 'none' }" />
        </div>
        <!-- Slider handle -->
        <div
          class="slider-handle absolute top-0 bottom-0 w-1 bg-primary cursor-col-resize z-10"
          :style="{ left: sliderPosition + '%' }"
          @mousedown="isDragging = true"
          @touchstart="isDragging = true"
        >
          <div class="slider-button absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-8 h-8 rounded-full bg-primary flex items-center justify-center text-white text-sm shadow-lg">
            ↔
          </div>
        </div>
      </div>

      <!-- Split mode -->
      <div v-else-if="compareMode === 'split'" class="h-full flex flex-col">
        <div class="flex-1 flex items-center justify-center border-b border-white/10">
          <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="max-w-full max-h-full object-contain" />
        </div>
        <div class="flex-1 flex items-center justify-center">
          <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="max-w-full max-h-full object-contain" />
        </div>
      </div>
    </div>

    <!-- File selection bar -->
    <div class="compare-files-bar flex items-center gap-4 px-4 py-2 bg-base-200/10">
      <div class="flex-1 flex items-center gap-2">
        <span class="text-white/40 text-xs">{{ $t('viewer.left_label') }}</span>
        <select v-model="leftIndex" class="select select-bordered select-xs text-white bg-base-300/50 max-w-[200px]">
          <option v-for="(file, i) in files" :key="file.path" :value="i" :disabled="i === rightIndex">
            {{ file.name }}
          </option>
        </select>
      </div>
      <div class="flex-1 flex items-center gap-2 justify-end">
        <span class="text-white/40 text-xs">{{ $t('viewer.right_label') }}</span>
        <select v-model="rightIndex" class="select select-bordered select-xs text-white bg-base-300/50 max-w-[200px]">
          <option v-for="(file, i) in files" :key="file.path" :value="i" :disabled="i === leftIndex">
            {{ file.name }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  files: { type: Array, default: () => [] },
  initialLeft: { type: Number, default: 0 },
  initialRight: { type: Number, default: 1 },
})

const emit = defineEmits(['close'])

const compareMode = ref('sidebyside')
const sliderPosition = ref(50)
const isDragging = ref(false)
const sliderContainer = ref(null)

const leftIndex = ref(props.initialLeft)
const rightIndex = ref(Math.min(props.initialRight, props.files.length - 1))

const leftFile = computed(() => props.files[leftIndex.value] || null)
const rightFile = computed(() => props.files[rightIndex.value] || null)

const modes = [
  { id: 'sidebyside', label: 'Side by Side' },
  { id: 'slider', label: 'Slider' },
  { id: 'split', label: 'Split' },
]

function getFileUrl(path) {
  return `asset://localhost/${encodeURI(path)}`
}

function onSliderMove(event) {
  if (!isDragging.value || !sliderContainer.value) return
  const rect = sliderContainer.value.getBoundingClientRect()
  const x = (event.clientX || event.touches?.[0]?.clientX || 0) - rect.left
  sliderPosition.value = Math.max(5, Math.min(95, (x / rect.width) * 100))
}

function handleMouseUp() {
  isDragging.value = false
}

function close() {
  emit('close')
}

onMounted(() => {
  document.addEventListener('mouseup', handleMouseUp)
  document.addEventListener('touchend', handleMouseUp)
})

onUnmounted(() => {
  document.removeEventListener('mouseup', handleMouseUp)
  document.removeEventListener('touchend', handleMouseUp)
})
</script>

<style scoped>
.slider-handle {
  transition: box-shadow 0.2s;
}
.slider-handle:hover {
  box-shadow: 0 0 10px rgba(255, 255, 255, 0.3);
}
.slider-button {
  pointer-events: none;
}
</style>