<template>
  <div class="compare-view fixed inset-0 z-40 bg-black/95 flex flex-col" @keydown.escape="close" tabindex="0" ref="rootRef">
    <!-- Top bar -->
    <div class="compare-header flex items-center justify-between px-4 py-2 bg-base-200/20 shrink-0">
      <h3 class="text-white/80 text-sm font-semibold">
        {{ files.length > 2 ? $t('viewer.compare_selected') + ` (${files.length})` : $t('viewer.compare_title', { left: leftFile?.name || '', right: rightFile?.name || '' }) }}
      </h3>
      <div class="flex items-center gap-4">
        <!-- Scale Range Slider -->
        <div class="flex items-center gap-2 select-none">
          <span class="text-white/60 text-xs">{{ $t('viewer.zoom') }}:</span>
          <input 
            type="range" 
            min="1" 
            max="4" 
            step="0.05" 
            v-model.number="scale" 
            class="range range-xs range-primary w-28"
          />
          <span class="text-white/80 font-mono text-xs w-10 text-right">{{ Math.round(scale * 100) }}%</span>
          <button 
            v-if="scale > 1" 
            class="btn btn-primary btn-xs rounded ml-1"
            @click="resetZoom"
          >
            {{ $t('viewer.reset') }}
          </button>
        </div>
        <button class="text-white/60 hover:text-white text-xl ml-2" @click="close">✕</button>
      </div>
    </div>

    <!-- Compare modes (only for 2 files) -->
    <div v-if="files.length === 2" class="compare-controls flex items-center justify-center gap-4 px-4 py-1 bg-base-200/10 shrink-0">
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
    <div 
      class="compare-area flex-1 relative overflow-hidden select-none" 
      ref="compareAreaRef"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @wheel="onWheel"
      :style="{ cursor: scale > 1 ? (isDragging ? 'grabbing' : 'grab') : 'default' }"
    >
      <!-- Grid Mode (for 3-6 files) -->
      <div v-if="files.length > 2" class="grid h-full w-full gap-2 p-2 bg-black" :class="gridClass">
        <div 
          v-for="file in files" 
          :key="file.path" 
          class="overflow-hidden relative flex items-center justify-center bg-zinc-900/40 rounded-lg border border-white/5"
        >
          <img 
            :src="getFileUrl(file.path)" 
            class="max-w-full max-h-full object-contain pointer-events-none select-none"
            :style="syncImageStyle"
          />
          <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[85%] pointer-events-none z-10">
            {{ file.name }}
          </div>
        </div>
      </div>

      <!-- Two files comparison modes -->
      <template v-else>
        <!-- Side-by-side mode -->
        <div v-if="compareMode === 'sidebyside'" class="flex h-full">
          <div class="flex-1 flex items-center justify-center border-r border-white/10 overflow-hidden relative bg-black">
            <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
            <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[80%] pointer-events-none z-10">{{ leftFile?.name }}</div>
          </div>
          <div class="flex-1 flex items-center justify-center overflow-hidden relative bg-black">
            <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
            <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[80%] pointer-events-none z-10">{{ rightFile?.name }}</div>
          </div>
        </div>

        <!-- Slider (curtain) mode -->
        <div v-else-if="compareMode === 'slider'" class="h-full relative overflow-hidden" ref="sliderContainer"
          @mousemove.self="onSliderMove" @touchmove.self="onSliderMove">
          <!-- Background (Right image) -->
          <div class="absolute inset-0 flex items-center justify-center bg-black">
            <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
            <div class="absolute bottom-2 right-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[40%] pointer-events-none z-10">{{ rightFile?.name }}</div>
          </div>
          <!-- Foreground (Left image) clip -->
          <div class="absolute inset-y-0 left-0 overflow-hidden bg-transparent pointer-events-none" :style="{ width: sliderPosition + '%' }">
            <div class="absolute inset-y-0 left-0 h-full flex items-center justify-center bg-black" :style="{ width: containerWidth + 'px' }">
              <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
              <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[40%] pointer-events-none z-10">{{ leftFile?.name }}</div>
            </div>
          </div>
          <!-- Slider handle -->
          <div
            class="slider-handle absolute top-0 bottom-0 w-1 bg-primary cursor-col-resize z-20"
            :style="{ left: sliderPosition + '%' }"
            @mousedown.stop="isDraggingSlider = true"
            @touchstart.stop="isDraggingSlider = true"
          >
            <div class="slider-button absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-8 h-8 rounded-full bg-primary flex items-center justify-center text-white text-sm shadow-lg pointer-events-none">
              ↔
            </div>
          </div>
        </div>

        <!-- Split mode -->
        <div v-else-if="compareMode === 'split'" class="h-full flex flex-col">
          <div class="flex-1 flex items-center justify-center border-b border-white/10 overflow-hidden relative bg-black">
            <img v-if="leftFile" :src="getFileUrl(leftFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
            <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[80%] pointer-events-none z-10">{{ leftFile?.name }}</div>
          </div>
          <div class="flex-1 flex items-center justify-center overflow-hidden relative bg-black">
            <img v-if="rightFile" :src="getFileUrl(rightFile.path)" class="max-w-full max-h-full object-contain pointer-events-none select-none" :style="syncImageStyle" />
            <div class="absolute bottom-2 left-2 bg-black/60 text-white text-[11px] px-2 py-1 rounded truncate max-w-[80%] pointer-events-none z-10">{{ rightFile?.name }}</div>
          </div>
        </div>
      </template>
    </div>

    <!-- File selection bar (only for 2 files) -->
    <div v-if="files.length === 2" class="compare-files-bar flex items-center gap-4 px-4 py-2 bg-base-200/10 shrink-0">
      <div class="flex-1 flex items-center gap-2">
        <span class="text-white/40 text-xs">{{ $t('viewer.left_label') }}</span>
        <select v-model="leftIndex" class="select select-bordered select-xs text-white bg-base-300/50 max-w-[200px]" @change="resetZoom">
          <option v-for="(file, i) in files" :key="file.path" :value="i" :disabled="i === rightIndex">
            {{ file.name }}
          </option>
        </select>
      </div>
      <div class="flex-1 flex items-center gap-2 justify-end">
        <span class="text-white/40 text-xs">{{ $t('viewer.right_label') }}</span>
        <select v-model="rightIndex" class="select select-bordered select-xs text-white bg-base-300/50 max-w-[200px]" @change="resetZoom">
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
import { getAssetSrc } from '@/common/utils'

const props = defineProps({
  files: { type: Array, default: () => [] },
  initialLeft: { type: Number, default: 0 },
  initialRight: { type: Number, default: 1 },
})

const emit = defineEmits(['close'])

const rootRef = ref(null)
const compareMode = ref('sidebyside')
const sliderPosition = ref(50)
const isDraggingSlider = ref(false)
const sliderContainer = ref(null)

const leftIndex = ref(props.initialLeft)
const rightIndex = ref(Math.min(props.initialRight, props.files.length - 1))

const leftFile = computed(() => props.files[leftIndex.value] || null)
const rightFile = computed(() => props.files[rightIndex.value] || null)

// Sync Zoom & Pan State
const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
const dragStart = { x: 0, y: 0 }

const compareAreaRef = ref(null)
const containerWidth = ref(800)
let resizeObserver = null

const modes = [
  { id: 'sidebyside', label: 'Side by Side' },
  { id: 'slider', label: 'Slider' },
  { id: 'split', label: 'Split' },
]

const syncImageStyle = computed(() => {
  return {
    transform: `translate(${translateX.value}px, ${translateY.value}px) scale(${scale.value})`,
    transformOrigin: 'center center'
  }
})

const gridClass = computed(() => {
  const count = props.files.length
  if (count <= 2) return 'grid-cols-2'
  if (count === 3) return 'grid-cols-3'
  if (count === 4) return 'grid-cols-2 grid-rows-2'
  return 'grid-cols-3 grid-rows-2'
})

function getFileUrl(path) {
  return getAssetSrc(path)
}

function onSliderMove(event) {
  if (!isDraggingSlider.value || !sliderContainer.value) return
  const rect = sliderContainer.value.getBoundingClientRect()
  const x = (event.clientX || event.touches?.[0]?.clientX || 0) - rect.left
  sliderPosition.value = Math.max(5, Math.min(95, (x / rect.width) * 100))
}

// Wheel handler for sync zooming
function onWheel(event) {
  event.preventDefault()
  const delta = event.deltaY < 0 ? 0.15 : -0.15
  const newScale = Math.max(1, Math.min(4, scale.value + delta))
  if (newScale === 1) {
    translateX.value = 0
    translateY.value = 0
  }
  scale.value = newScale
}

// Mouse pan handlers
function onMouseDown(event) {
  if (isDraggingSlider.value) return
  if (scale.value <= 1) return
  if (event.button !== 0) return
  isDragging.value = true
  dragStart.x = event.clientX - translateX.value
  dragStart.y = event.clientY - translateY.value
  event.preventDefault()
}

function onMouseMove(event) {
  if (isDraggingSlider.value) {
    onSliderMove(event)
    return
  }
  if (!isDragging.value) return
  translateX.value = event.clientX - dragStart.x
  translateY.value = event.clientY - dragStart.y
}

function onMouseUp() {
  isDragging.value = false
  isDraggingSlider.value = false
}

function resetZoom() {
  scale.value = 1
  translateX.value = 0
  translateY.value = 0
}

function close() {
  emit('close')
}

// Global Escape Key Listener
function handleGlobalKeyDown(event) {
  if (event.key === 'Escape') {
    close()
  }
}

onMounted(() => {
  document.addEventListener('mouseup', onMouseUp)
  document.addEventListener('touchend', onMouseUp)
  document.addEventListener('keydown', handleGlobalKeyDown)

  if (compareAreaRef.value) {
    containerWidth.value = compareAreaRef.value.clientWidth || 800
    resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerWidth.value = entry.contentRect.width
      }
    })
    resizeObserver.observe(compareAreaRef.value)
  }

  // Focus root element to capture escape key
  rootRef.value?.focus()
})

onUnmounted(() => {
  document.removeEventListener('mouseup', onMouseUp)
  document.removeEventListener('touchend', onMouseUp)
  document.removeEventListener('keydown', handleGlobalKeyDown)
  if (resizeObserver) resizeObserver.disconnect()
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
.compare-view:focus {
  outline: none;
}
</style>