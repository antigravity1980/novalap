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
        <div v-if="isCurrentImage" class="flex items-center gap-6 bg-white/5 px-4 py-1.5 rounded-lg border border-white/5">
          <!-- Saturation Slider -->
          <div class="flex items-center gap-2">
            <span class="text-white/50 text-[11px] font-bold uppercase tracking-wider">{{ $t('viewer.saturation') || 'Насыщенность' }}:</span>
            <input
              type="range"
              min="0"
              max="2"
              step="0.05"
              v-model.number="saturation"
              @change="triggerAutoSave"
              class="range range-xs range-primary w-24"
            />
            <span class="text-white/80 font-mono text-xs w-8 text-right">{{ Math.round(saturation * 100) }}%</span>
          </div>

          <!-- Gamma Slider -->
          <div class="flex items-center gap-2 border-l border-white/10 pl-6">
            <span class="text-white/50 text-[11px] font-bold uppercase tracking-wider">{{ $t('viewer.gamma') || 'Гамма' }}:</span>
            <input
              type="range"
              min="0.2"
              max="2.5"
              step="0.05"
              v-model.number="gamma"
              @change="triggerAutoSave"
              class="range range-xs range-primary w-24"
            />
            <span class="text-white/80 font-mono text-xs w-8 text-right">{{ Math.round(gamma * 100) }}%</span>
          </div>

          <!-- Undo Button Indicator -->
          <button
            v-if="undoStack.length > 0"
            class="btn btn-primary btn-xs rounded ml-2"
            @click="handleUndo"
            title="Отменить изменения (Ctrl+Z)"
          >
            ↩ {{ $t('viewer.undo') || 'Отменить' }}
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
        class="absolute left-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white text-4xl z-10 bg-black/40 hover:bg-black/60 rounded-full w-12 h-12 flex items-center justify-center transition-all duration-150"
        @click="prev"
      >
        ‹
      </button>
      <button
        v-if="currentIndex < files.length - 1"
        class="absolute right-4 top-1/2 -translate-y-1/2 text-white/60 hover:text-white text-4xl z-10 bg-black/40 hover:bg-black/60 rounded-full w-12 h-12 flex items-center justify-center transition-all duration-150"
        @click="next"
      >
        ›
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
        >
          <img
            ref="imageRef"
            :src="currentFileUrl"
            class="max-w-[85vw] max-h-[72vh] rounded select-none pointer-events-none"
            :style="{
              filter: `saturate(${saturation})`
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
              @mousedown.self="startCropDrag"
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
    </div>
  </teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { getAssetSrc } from '@/common/utils'

const props = defineProps({
  visible: { type: Boolean, default: false },
  files: { type: Array, default: () => [] },
  initialIndex: { type: Number, default: 0 },
})

const emit = defineEmits(['close', 'update:visible', 'saved'])

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
let currentSaving = false

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

watch(() => props.visible, (val) => {
  if (val) {
    currentIndex.value = props.initialIndex
    resetEditState()
    nextTick(() => overlayRef.value?.focus())
    document.addEventListener('keydown', handleGlobalKeyDown)
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
  imageLoaded.value = false
  cacheBuster.value = 0
}

function onImageLoad() {
  imageLoaded.value = true
  if (imageRef.value) {
    const width = imageRef.value.clientWidth
    const height = imageRef.value.clientHeight
    // Center a 80% size crop box
    crop.x = width * 0.1
    crop.y = height * 0.1
    crop.width = width * 0.8
    crop.height = height * 0.8
  }
}

// Mouse dragging handlers for crop box
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

function onMouseMove(event) {
  if (!imageLoaded.value || !imageRef.value) return

  const width = imageRef.value.clientWidth
  const height = imageRef.value.clientHeight

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
  if (isDragging.value || isResizing.value) {
    isDragging.value = false
    isResizing.value = false
    dragHandle.value = null
    triggerAutoSave()
  }
}

// Auto Save mechanism
async function triggerAutoSave() {
  if (!currentFile.value || currentSaving) return
  currentSaving = true

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

  try {
    const path = currentFile.value.path

    if (!isTauri) {
      // Browser fallback simulation
      undoStack.value.push({
        bytes: new Uint8Array(),
        crop: { ...crop },
        saturation: saturation.value,
        gamma: gamma.value
      })
      cacheBuster.value = Date.now()
      emit('saved')
      currentSaving = false
      return
    }

    // 1. Read original bytes first if not saved in memory yet
    if (!originalBytes) {
      originalBytes = await readFile(path)
    }

    // 2. Save current file state on disk to undoStack
    const currentBytes = await readFile(path)
    undoStack.value.push({
      bytes: currentBytes,
      crop: { ...crop },
      saturation: saturation.value,
      gamma: gamma.value
    })

    if (undoStack.value.length > 25) {
      undoStack.value.shift()
    }

    // 3. Write originalBytes to a temp file
    const tempPath = path + ".edit_temp"
    await writeFile(tempPath, originalBytes)

    // 4. Calculate coordinates based on image naturalWidth/Height
    const displayWidth = imageRef.value.clientWidth
    const displayHeight = imageRef.value.clientHeight
    
    // We need natural dimensions. We load a quick Image instance to get it.
    const img = new Image()
    img.crossOrigin = 'anonymous'
    await new Promise((resolve, reject) => {
      img.onload = resolve
      img.onerror = reject
      img.src = getAssetSrc(tempPath)
    })

    const scaleX = img.naturalWidth / displayWidth
    const scaleY = img.naturalHeight / displayHeight

    const cropX = crop.x * scaleX
    const cropY = crop.y * scaleY
    const cropW = crop.width * scaleX
    const cropH = crop.height * scaleY

    // 5. Invoke edit_image
    const cropData = {
      x: Math.round(cropX),
      y: Math.round(cropY),
      width: Math.round(cropW),
      height: Math.round(cropH),
    }

    const params = {
      sourceFilePath: tempPath,
      destFilePath: path, // Overwrite original
      outputFormat: currentFile.value.extension?.toLowerCase() || 'jpg',
      orientation: 1,
      flipHorizontal: false,
      flipVertical: false,
      rotate: 0,
      crop: cropData,
      resize: { width: null, height: null },
      quality: 95,
      saturation: saturation.value,
      gamma: gamma.value,
    }

    const success = await invoke('edit_image', { params })

    // Delete temporary file
    await invoke('delete_file_system', { path: tempPath }).catch(() => {})

    if (success) {
      cacheBuster.value = Date.now()
      emit('saved')
    }
  } catch (err) {
    console.error('Auto save failed:', err)
  } finally {
    currentSaving = false
  }
}

// Undo changes
async function handleUndo() {
  if (undoStack.value.length === 0) return
  const prevState = undoStack.value.pop()

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

  try {
    if (!isTauri) {
      // Browser mode fallback
      saturation.value = prevState.saturation
      gamma.value = prevState.gamma
      Object.assign(crop, prevState.crop)
      cacheBuster.value = Date.now()
      emit('saved')
      return
    }

    // Write previous bytes back to original path
    await writeFile(currentFile.value.path, prevState.bytes)

    // Restore UI variables
    saturation.value = prevState.saturation
    gamma.value = prevState.gamma
    Object.assign(crop, prevState.crop)

    // Update screen
    cacheBuster.value = Date.now()
    emit('saved')
  } catch (err) {
    console.error('Undo failed:', err)
  }
}

// Key events (Ctrl+Z, arrows, space, escape)
function handleKeyDown(event) {
  if (!props.visible) return

  // Ctrl+Z Undo
  if (event.ctrlKey && event.key === 'z') {
    event.preventDefault()
    handleUndo()
    return
  }

  switch (event.key) {
    case 'Escape':
    case ' ':
      event.preventDefault()
      close()
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