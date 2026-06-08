<template>
  <teleport to="body">
    <div
      v-if="visible"
      class="quick-look-overlay fixed inset-0 z-50 bg-black/95 flex flex-col select-none"
      @click.self="close"
      @keydown="handleKeyDown"
      tabindex="0"
      ref="overlayRef"
    >
      <!-- Top bar with sliders and controls -->
      <div class="quick-look-header flex items-center justify-between px-6 py-2.5 bg-zinc-950/80 border-b border-white/5 shrink-0 z-10 text-white">
        <!-- File counter and name -->
        <div class="flex items-center gap-3">
          <span class="text-white/60 text-xs font-semibold bg-white/10 px-2 py-0.5 rounded">
            {{ currentIndex + 1 }} / {{ files.length }}
          </span>
          <span class="text-sm font-semibold truncate max-w-[200px]" :title="currentFile?.name">
            {{ currentFile?.name }}
          </span>
        </div>

        <!-- Adjustments Sliders (only for images) -->
        <div v-if="isCurrentImage" class="flex items-center gap-4 bg-white/5 px-4 py-1.5 rounded-lg border border-white/5">
          <!-- Saturation Slider -->
          <div class="flex items-center gap-2">
            <span class="text-white/50 text-[11px] font-bold uppercase tracking-wider">{{ $t('viewer.saturation') }}:</span>
            <input
              type="range"
              min="0"
              max="2"
              step="0.05"
              v-model.number="saturation"
              class="range range-xs range-primary w-24"
            />
            <span class="text-white/80 font-mono text-xs w-8 text-right">{{ Math.round(saturation * 100) }}%</span>
          </div>

          <!-- Gamma Slider -->
          <div class="flex items-center gap-2 border-l border-white/10 pl-4">
            <span class="text-white/50 text-[11px] font-bold uppercase tracking-wider">{{ $t('viewer.gamma') }}:</span>
            <input
              type="range"
              min="0.2"
              max="2.5"
              step="0.05"
              v-model.number="gamma"
              class="range range-xs range-primary w-24"
            />
            <span class="text-white/80 font-mono text-xs w-8 text-right">{{ Math.round(gamma * 100) }}%</span>
          </div>

          <!-- Flip Horizontally -->
          <button
            class="btn btn-ghost btn-xs text-white/75 hover:text-white hover:bg-white/10 flex items-center gap-1 px-2 py-1 rounded border border-white/10"
            @click="toggleFlipHorizontal"
            :title="$t('viewer.flip_horizontal')"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
            </svg>
            <span>{{ $t('viewer.flip_horizontal_btn') }}</span>
          </button>

          <!-- Reset Button -->
          <button
            v-if="isModified"
            class="btn btn-ghost btn-xs text-white/60 hover:text-white hover:bg-white/10 flex items-center gap-1 px-2 py-1 rounded border border-white/10 ml-2"
            @click="resetAllEdits"
            title="Сбросить все изменения"
          >
            <span>↩</span>
            <span>{{ $t('viewer.reset') }}</span>
          </button>

          <div class="divider divider-horizontal h-4 bg-white/10 mx-1"></div>

          <!-- Cancel (Cross) -->
          <button
            class="btn btn-ghost btn-xs text-red-400 hover:text-red-300 hover:bg-red-500/10 flex items-center gap-1 px-2 py-1 rounded border border-red-500/20"
            @click="close"
            :title="$t('viewer.cancel') + ' (Esc)'"
          >
            <span>✕</span>
            <span>{{ $t('viewer.cancel') }}</span>
          </button>

          <!-- Save (Checkmark) -->
          <button
            class="btn btn-primary btn-xs flex items-center gap-1 px-2.5 py-1 rounded shadow shadow-primary/20"
            :disabled="currentSaving"
            @click="saveAndClose"
            :title="$t('viewer.save_confirm') + ' (Enter)'"
          >
            <span v-if="currentSaving" class="loading loading-spinner loading-xs"></span>
            <span v-else>✓</span>
            <span>{{ $t('viewer.save_confirm') }}</span>
          </button>
        </div>

        <!-- Close action -->
        <button class="text-white/60 hover:text-white text-2xl" @click="close">
          ✕
        </button>
      </div>

      <!-- Navigation arrows -->
      <button
        v-if="currentIndex > 0"
        class="absolute left-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white z-10 bg-black/40 hover:bg-black/60 rounded-full w-12 h-12 flex items-center justify-center transition-all duration-150 animate-fade-in"
        @click="prev"
        title="Предыдущее изображение"
      >
        <svg class="w-6 h-6 stroke-current" fill="none" stroke-width="2.5" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
        </svg>
      </button>
      <button
        v-if="currentIndex < files.length - 1"
        class="absolute right-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white z-10 bg-black/40 hover:bg-black/60 rounded-full w-12 h-12 flex items-center justify-center transition-all duration-150 animate-fade-in"
        @click="next"
        title="Следующее изображение"
      >
        <svg class="w-6 h-6 stroke-current" fill="none" stroke-width="2.5" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
        </svg>
      </button>

      <!-- Main workspace -->
      <div class="flex-1 flex items-center justify-center p-6 overflow-hidden" ref="workspaceRef">
        <!-- Interactive Crop Container for images -->
        <div 
          v-if="isCurrentImage" 
          class="crop-container relative select-none" 
          ref="cropContainer"
          @mousemove="onMouseMove"
          @mouseup="onMouseUp"
          @mouseleave="onMouseUp"
          @mousedown="handleMouseDownContainer"
          @wheel="handleWheel"
          :style="{
            transform: `translate(${panOffset.x}px, ${panOffset.y}px) scale(${zoomScale})`,
            transformOrigin: 'center center'
          }"
        >
          <img
            ref="imageRef"
            :src="currentFileUrl"
            class="max-w-[85vw] max-h-[72vh] rounded select-none pointer-events-none transition-transform duration-100"
            :style="{
              filter: `url(#gamma-filter-ql) saturate(${saturation})`,
              transform: flipHorizontal ? 'scaleX(-1)' : ''
            }"
            @load="onImageLoad"
            @error="onImageError"
          />

          <!-- Crop overlay overlay border -->
          <div v-if="imageLoaded" class="crop-overlay absolute inset-0 pointer-events-none">
            <!-- Dark overlay with transparent center -->
            <div class="absolute inset-0 bg-black/60"
              :style="{
                clipPath: `polygon(
                  0% 0%,
                  100% 0%,
                  100% 100%,
                  0% 100%,
                  0% 0%,
                  ${crop.x}px ${crop.y}px,
                  ${crop.x}px ${crop.y + crop.height}px,
                  ${crop.x + crop.width}px ${crop.y + crop.height}px,
                  ${crop.x + crop.width}px ${crop.y}px,
                  ${crop.x}px ${crop.y}px
                )`
              }"
            />
            <!-- Crop border -->
            <div class="absolute border-2 border-primary pointer-events-auto cursor-move"
              :style="{
                left: crop.x + 'px',
                top: crop.y + 'px',
                width: crop.width + 'px',
                height: crop.height + 'px',
              }"
              @mousedown="handleMouseDownContainer"
            >
              <!-- Drag handles -->
              <div v-for="handle in handles" :key="handle.name"
                class="crop-handle absolute w-3 h-3 bg-primary border border-white rounded-sm"
                :style="handle.style"
                @mousedown.stop="startHandleDrag(handle.name, $event)"
              />
            </div>
          </div>
        </div>

        <!-- Video player -->
        <video
          v-else-if="isCurrentVideo"
          :src="currentFileUrl"
          class="max-w-full max-h-[75vh] rounded"
          controls
          autoplay
        ></video>

        <!-- Fallback -->
        <div v-else class="text-white/50 text-lg">
          {{ $t('viewer.unsupported_file_type', { ext: currentFile?.extension || '' }) }}
        </div>
      </div>

      <!-- File info bottom bar -->
      <div class="file-info-bar py-3 bg-zinc-950/50 text-white/50 text-xs flex items-center justify-center gap-6 shrink-0 border-t border-white/5">
        <span class="font-medium text-white/70">{{ currentFile?.name }}</span>
        <span v-if="currentFile?.resolution" class="font-mono bg-white/5 px-2 py-0.5 rounded">
          {{ currentFile.resolution.width }}x{{ currentFile.resolution.height }}
        </span>
        <span class="font-mono bg-white/5 px-2 py-0.5 rounded">{{ formatFileSize(currentFile?.size) }}</span>
        <span v-if="currentFile?.ai_source" class="text-primary/80 font-semibold uppercase tracking-wider bg-primary/10 px-2 py-0.5 rounded">
          {{ currentFile.ai_source }}
        </span>
      </div>
      <!-- SVG Gamma Filter -->
      <svg style="position: absolute; width: 0; height: 0; pointer-events: none;">
        <filter id="gamma-filter-ql">
          <feComponentTransfer>
            <feFuncR type="gamma" :exponent="gamma !== 0 ? 1 / gamma : 1" />
            <feFuncG type="gamma" :exponent="gamma !== 0 ? 1 / gamma : 1" />
            <feFuncB type="gamma" :exponent="gamma !== 0 ? 1 / gamma : 1" />
          </feComponentTransfer>
        </filter>
      </svg>
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { getAssetSrc } from '@/common/utils'
import { useUIStore } from '@/stores/uiStore'

const props = defineProps({
  visible: { type: Boolean, default: false },
  files: { type: Array, default: () => [] },
  initialIndex: { type: Number, default: 0 },
})

const emit = defineEmits(['close', 'update:visible', 'saved'])

const uiStore = useUIStore()

const overlayRef = ref(null)
const workspaceRef = ref(null)
const imageRef = ref(null)
const cropContainer = ref(null)
const currentIndex = ref(0)
const cacheBuster = ref(0)

// Quick edit states
const saturation = ref(1.0)
const gamma = ref(1.0)
const imageLoaded = ref(false)

// Undo stack in memory
const undoStack = ref([])
let originalBytes = null
const currentSaving = ref(false)

const crop = reactive({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
})

const isDragging = ref(false)
const isResizing = ref(false)
const dragHandle = ref(null)
const dragStart = reactive({ x: 0, y: 0 })

const handles = computed(() => [
  { name: 'nw', style: { left: '-6px', top: '-6px', cursor: 'nw-resize' } },
  { name: 'ne', style: { right: '-6px', top: '-6px', cursor: 'ne-resize' } },
  { name: 'sw', style: { left: '-6px', bottom: '-6px', cursor: 'sw-resize' } },
  { name: 'se', style: { right: '-6px', bottom: '-6px', cursor: 'se-resize' } },
  { name: 'n', style: { left: '50%', top: '-6px', marginLeft: '-6px', cursor: 'n-resize' } },
  { name: 's', style: { left: '50%', bottom: '-6px', marginLeft: '-6px', cursor: 's-resize' } },
  { name: 'e', style: { right: '-6px', top: '50%', marginTop: '-6px', cursor: 'e-resize' } },
  { name: 'w', style: { left: '-6px', top: '50%', marginTop: '-6px', cursor: 'w-resize' } },
])

const currentFile = computed(() => props.files[currentIndex.value] || null)
const currentFileUrl = computed(() => {
  if (!currentFile.value) return ''
  const base = getAssetSrc(currentFile.value.path)
  return cacheBuster.value ? `${base}?t=${cacheBuster.value}` : base
})

const isCurrentImage = computed(() => {
  const ext = currentFile.value?.extension?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'avif', 'jxl', 'svg', 'ico'].includes(ext)
})

const isCurrentVideo = computed(() => {
  const ext = currentFile.value?.extension?.toLowerCase()
  return ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mpeg', '3gp'].includes(ext)
})

const flipHorizontal = ref(false)
const zoomScale = ref(1.0)
const panOffset = reactive({ x: 0, y: 0 })
const isPanning = ref(false)
const panStart = reactive({ x: 0, y: 0 })
let hasPendingSave = false

watch(() => props.visible, (val) => {
  if (val) {
    currentIndex.value = props.initialIndex
    resetEditState()
    nextTick(() => overlayRef.value?.focus())
    // Add keydown listener with a small delay so we do not catch the Space bar keydown that opened us
    setTimeout(() => {
      if (props.visible) {
        document.addEventListener('keydown', handleGlobalKeyDown)
      }
    }, 50)
  } else {
    document.removeEventListener('keydown', handleGlobalKeyDown)
  }
})

watch(currentIndex, () => {
  resetEditState()
})

function resetEditState() {
  originalBytes = null
  undoStack.value = []
  saturation.value = 1.0
  gamma.value = 1.0
  flipHorizontal.value = false
  zoomScale.value = 1.0
  panOffset.x = 0
  panOffset.y = 0
  imageLoaded.value = false
  cacheBuster.value = 0
  hasPendingSave = false
}

function onImageLoad() {
  imageLoaded.value = true
  if (imageRef.value) {
    const width = imageRef.value.clientWidth
    const height = imageRef.value.clientHeight
    crop.x = 0
    crop.y = 0
    crop.width = width
    crop.height = height
  }
}

// Mouse dragging handlers for crop box and panning
function startCropDrag(event) {
  isDragging.value = true
  isResizing.value = false
  dragStart.x = event.clientX
  dragStart.y = event.clientY
  event.preventDefault()
}

function startHandleDrag(handleName, event) {
  isResizing.value = true
  isDragging.value = false
  dragHandle.value = handleName
  dragStart.x = event.clientX
  dragStart.y = event.clientY
  event.preventDefault()
}

function handleMouseDownContainer(event) {
  if (event.button !== 0) return
  if (event.target.closest('.crop-handle')) return

  const isClickingCropBox = event.target.closest('.border-primary')
  if (isClickingCropBox && zoomScale.value === 1.0) {
    startCropDrag(event)
  } else {
    isPanning.value = true
    panStart.x = event.clientX - panOffset.x
    panStart.y = event.clientY - panOffset.y
    event.preventDefault()
  }
}

function handleWheel(event) {
  event.preventDefault()
  const delta = event.deltaY < 0 ? 0.1 : -0.1
  const prevScale = zoomScale.value
  let newScale = prevScale + delta * prevScale
  newScale = Math.max(1.0, Math.min(10.0, newScale))

  if (newScale === 1.0) {
    panOffset.x = 0
    panOffset.y = 0
  }

  zoomScale.value = newScale
}

function toggleFlipHorizontal() {
  flipHorizontal.value = !flipHorizontal.value
}

function onMouseMove(event) {
  if (!imageLoaded.value || !imageRef.value) return

  const width = imageRef.value.clientWidth
  const height = imageRef.value.clientHeight

  if (isPanning.value) {
    panOffset.x = event.clientX - panStart.x
    panOffset.y = event.clientY - panStart.y
    return
  }

  if (isDragging.value && !isResizing.value) {
    const dx = event.clientX - dragStart.x
    const dy = event.clientY - dragStart.y
    
    // Move crop and clamp
    crop.x = Math.max(0, Math.min(width - crop.width, crop.x + dx))
    crop.y = Math.max(0, Math.min(height - crop.height, crop.y + dy))

    dragStart.x = event.clientX
    dragStart.y = event.clientY
  }

  if (isResizing.value && dragHandle.value) {
    const dx = event.clientX - dragStart.x
    const dy = event.clientY - dragStart.y
    resizeCrop(dragHandle.value, dx, dy, width, height)
    dragStart.x = event.clientX
    dragStart.y = event.clientY
  }
}

function resizeCrop(handle, dx, dy, width, height) {
  switch (handle) {
    case 'nw':
      crop.x = Math.max(0, Math.min(crop.x + crop.width - 20, crop.x + dx))
      crop.y = Math.max(0, Math.min(crop.y + crop.height - 20, crop.y + dy))
      crop.width = crop.width - dx
      crop.height = crop.height - dy
      break
    case 'ne':
      crop.y = Math.max(0, Math.min(crop.y + crop.height - 20, crop.y + dy))
      crop.width = Math.max(20, Math.min(width - crop.x, crop.width + dx))
      crop.height = crop.height - dy
      break
    case 'sw':
      crop.x = Math.max(0, Math.min(crop.x + crop.width - 20, crop.x + dx))
      crop.width = crop.width - dx
      crop.height = Math.max(20, Math.min(height - crop.y, crop.height + dy))
      break
    case 'se':
      crop.width = Math.max(20, Math.min(width - crop.x, crop.width + dx))
      crop.height = Math.max(20, Math.min(height - crop.y, crop.height + dy))
      break
    case 'n':
      crop.y = Math.max(0, Math.min(crop.y + crop.height - 20, crop.y + dy))
      crop.height = crop.height - dy
      break
    case 's':
      crop.height = Math.max(20, Math.min(height - crop.y, crop.height + dy))
      break
    case 'e':
      crop.width = Math.max(20, Math.min(width - crop.x, crop.width + dx))
      break
    case 'w':
      crop.x = Math.max(0, Math.min(crop.x + crop.width - 20, crop.x + dx))
      crop.width = crop.width - dx
      break
  }
  // Clamp width / height positive
  crop.width = Math.max(20, crop.width)
  crop.height = Math.max(20, crop.height)
}

function onMouseUp() {
  if (isPanning.value) {
    isPanning.value = false
  }
  if (isDragging.value || isResizing.value) {
    isDragging.value = false
    isResizing.value = false
    dragHandle.value = null
  }
}

// Computed modified state helper
const isModified = computed(() => {
  if (!imageLoaded.value || !imageRef.value) return false
  const displayWidth = imageRef.value.clientWidth
  const displayHeight = imageRef.value.clientHeight
  const isCropModified = Math.abs(crop.x) > 1 || 
                         Math.abs(crop.y) > 1 || 
                         Math.abs(crop.width - displayWidth) > 2 || 
                         Math.abs(crop.height - displayHeight) > 2
  return saturation.value !== 1.0 || gamma.value !== 1.0 || flipHorizontal.value || isCropModified
})

// Reset edits manually
function resetAllEdits() {
  saturation.value = 1.0
  gamma.value = 1.0
  flipHorizontal.value = false
  zoomScale.value = 1.0
  panOffset.x = 0
  panOffset.y = 0
  if (imageRef.value) {
    crop.x = 0
    crop.y = 0
    crop.width = imageRef.value.clientWidth
    crop.height = imageRef.value.clientHeight
  }
}

// Explicit Save changes and close
async function saveAndClose() {
  if (!currentFile.value) return
  if (currentSaving.value) return
  currentSaving.value = true

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

  try {
    const path = currentFile.value.path

    if (!isTauri) {
      // Browser fallback simulation
      close()
      return
    }

    // 1. Calculate coordinates based on image naturalWidth/Height
    const displayWidth = imageRef.value.clientWidth
    const displayHeight = imageRef.value.clientHeight
    
    // We need natural dimensions. We load a quick Image instance to get it.
    const img = new Image()
    img.crossOrigin = 'anonymous'
    await new Promise((resolve, reject) => {
      img.onload = resolve
      img.onerror = reject
      img.src = getAssetSrc(path)
    })

    const scaleX = img.naturalWidth / displayWidth
    const scaleY = img.naturalHeight / displayHeight

    // Check if the crop box is modified
    const isCropModified = Math.abs(crop.x) > 1 || 
                           Math.abs(crop.y) > 1 || 
                           Math.abs(crop.width - displayWidth) > 2 || 
                           Math.abs(crop.height - displayHeight) > 2

    let cropData = { x: 0, y: 0, width: 0, height: 0 }
    if (isCropModified) {
      cropData = {
        x: Math.round(crop.x * scaleX),
        y: Math.round(crop.y * scaleY),
        width: Math.round(crop.width * scaleX),
        height: Math.round(crop.height * scaleY),
      }
    }

    const params = {
      sourceFilePath: path,
      destFilePath: path, // Overwrite original directly
      outputFormat: currentFile.value.extension?.toLowerCase() || 'jpg',
      orientation: 1,
      flipHorizontal: flipHorizontal.value,
      flipVertical: false,
      rotate: 0,
      crop: cropData,
      resize: { width: null, height: null },
      quality: 95,
      saturation: saturation.value,
      gamma: gamma.value,
    }

    const success = await invoke('edit_image', { params })

     if (success) {
      uiStore.updateFileVersion(params.destFilePath)
      cacheBuster.value = Date.now()
      emit('saved')
      close()
    } else {
      alert('Не удалось сохранить изменения')
    }
  } catch (err) {
    console.error('Save failed:', err)
    alert('Ошибка при сохранении: ' + err)
  } finally {
    currentSaving.value = false
  }
}

// Key events (Ctrl+Z, arrows, space, escape)
function handleKeyDown(event) {
  if (!props.visible) return

  switch (event.key) {
    case 'Escape':
      event.preventDefault()
      close()
      break
    case 'Enter':
      event.preventDefault()
      saveAndClose()
      break
    case 'ArrowLeft':
      event.preventDefault()
      prev()
      break
    case 'ArrowRight':
      event.preventDefault()
      next()
      break
  }
}

// Global key down handler backup
function handleGlobalKeyDown(event) {
  handleKeyDown(event)
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
}
</script>

<style scoped>
.quick-look-overlay {
  backdrop-filter: blur(4px);
}
.quick-look-overlay:focus {
  outline: none;
}
.crop-container {
  position: relative;
  display: inline-block;
}
.crop-overlay {
  pointer-events: none;
}
.crop-handle {
  pointer-events: all;
  z-index: 10;
}
.crop-handle:hover {
  transform: scale(1.25);
}
</style>