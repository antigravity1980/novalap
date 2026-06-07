<template>
  <div class="quick-crop fixed inset-0 z-40 bg-black/90 flex flex-col" @keydown.escape="cancel">
    <!-- Header -->
    <div class="crop-header flex items-center justify-between px-4 py-2 bg-base-200/20 text-white">
      <h3 class="text-sm">{{ $t('viewer.crop_title', { name: file?.name || '' }) }}</h3>
      <div class="flex items-center gap-2">
        <button class="btn btn-ghost btn-xs text-white/60 hover:text-white" @click="resetCrop">
          {{ $t('viewer.reset') }}
        </button>
        <button class="btn btn-primary btn-xs" @click="saveCrop" :disabled="saving">
          {{ saving ? $t('viewer.saving') : $t('viewer.save_as') }}
        </button>
        <button class="text-white/60 hover:text-white text-xl ml-2" @click="cancel">✕</button>
      </div>
    </div>

    <!-- Crop area -->
    <div class="crop-area flex-1 flex items-center justify-center overflow-hidden">
      <div class="crop-container relative" ref="cropContainer"
        @mousedown="onMouseDown"
        @mousemove="onMouseMove"
        @mouseup="onMouseUp"
        @mouseleave="onMouseUp"
      >
        <img
          ref="imageRef"
          :src="imageUrl"
          class="max-w-[90vw] max-h-[80vh]"
          :style="{ transform: `scale(${zoom})` }"
          @load="onImageLoad"
        />

        <!-- Crop overlay -->
        <div v-if="imageLoaded" class="crop-overlay absolute inset-0 pointer-events-none">
          <!-- Darkened areas -->
          <div class="absolute inset-0 bg-black/50"
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
          <div class="absolute border-2 border-primary"
            :style="{
              left: crop.x + 'px',
              top: crop.y + 'px',
              width: crop.width + 'px',
              height: crop.height + 'px',
            }"
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
    </div>

    <!-- Controls -->
    <div class="crop-controls flex items-center justify-center gap-4 px-4 py-2 bg-base-200/20 text-white/60 text-xs">
      <div class="flex items-center gap-2">
        <span>{{ $t('viewer.zoom') }}</span>
        <input type="range" min="0.1" max="3" step="0.1" v-model.number="zoom" class="range range-xs w-20" />
        <span>{{ Math.round(zoom * 100) }}%</span>
      </div>
      <div class="flex items-center gap-2">
        <span>{{ $t('viewer.aspect') }}</span>
        <select v-model="aspectRatio" class="select select-bordered select-xs bg-base-300/50 text-white">
          <option value="free">{{ $t('viewer.aspect_options.free') }}</option>
          <option value="1:1">1:1</option>
          <option value="4:3">4:3</option>
          <option value="16:9">16:9</option>
          <option value="3:2">3:2</option>
          <option value="9:16">{{ $t('viewer.aspect_options.portrait_9_16') }}</option>
        </select>
      </div>
      <span>{{ Math.round(crop.width) }} × {{ Math.round(crop.height) }}px</span>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { getAssetSrc } from '@/common/utils'

const props = defineProps({
  file: { type: Object, default: null },
})

const emit = defineEmits(['close', 'saved'])

const imageRef = ref(null)
const cropContainer = ref(null)
const imageUrl = computed(() => props.file ? getAssetSrc(props.file.path) : '')
const imageLoaded = ref(false)
const saving = ref(false)

const zoom = ref(1)
const aspectRatio = ref('free')
const isDragging = ref(false)
const isResizing = ref(false)
const dragHandle = ref(null)
const dragStart = reactive({ x: 0, y: 0 })

const crop = reactive({
  x: 50,
  y: 50,
  width: 300,
  height: 300,
})

const handles = computed(() => {
  const s = 12 // handle size
  return [
    { name: 'nw', style: { left: '-6px', top: '-6px', cursor: 'nw-resize' } },
    { name: 'ne', style: { right: '-6px', top: '-6px', cursor: 'ne-resize' } },
    { name: 'sw', style: { left: '-6px', bottom: '-6px', cursor: 'sw-resize' } },
    { name: 'se', style: { right: '-6px', bottom: '-6px', cursor: 'se-resize' } },
    { name: 'n', style: { left: '50%', top: '-6px', marginLeft: '-6px', cursor: 'n-resize' } },
    { name: 's', style: { left: '50%', bottom: '-6px', marginLeft: '-6px', cursor: 's-resize' } },
    { name: 'e', style: { right: '-6px', top: '50%', marginTop: '-6px', cursor: 'e-resize' } },
    { name: 'w', style: { left: '-6px', top: '50%', marginTop: '-6px', cursor: 'w-resize' } },
  ]
})

function onImageLoad() {
  imageLoaded.value = true
  if (imageRef.value) {
    const rect = imageRef.value.getBoundingClientRect()
    crop.x = rect.width * 0.1
    crop.y = rect.height * 0.1
    crop.width = rect.width * 0.8
    crop.height = rect.height * 0.8
  }
}

function onMouseDown(event) {
  isDragging.value = true
  dragStart.x = event.clientX
  dragStart.y = event.clientY
}

function onMouseMove(event) {
  if (!imageLoaded.value || !cropContainer.value) return

  if (isDragging.value && !isResizing.value) {
    const dx = event.clientX - dragStart.x
    const dy = event.clientY - dragStart.y
    crop.x += dx
    crop.y += dy
    dragStart.x = event.clientX
    dragStart.y = event.clientY
  }

  if (isResizing.value && dragHandle.value) {
    const dx = event.clientX - dragStart.x
    const dy = event.clientY - dragStart.y
    resizeCrop(dragHandle.value, dx, dy)
    dragStart.x = event.clientX
    dragStart.y = event.clientY
  }
}

function onMouseUp() {
  isDragging.value = false
  isResizing.value = false
  dragHandle.value = null
}

function startHandleDrag(handleName, event) {
  isResizing.value = true
  isDragging.value = false
  dragHandle.value = handleName
  dragStart.x = event.clientX
  dragStart.y = event.clientY
}

function resizeCrop(handle, dx, dy) {
  switch (handle) {
    case 'nw': crop.x += dx; crop.y += dy; crop.width -= dx; crop.height -= dy; break
    case 'ne': crop.y += dy; crop.width += dx; crop.height -= dy; break
    case 'sw': crop.x += dx; crop.width -= dx; crop.height += dy; break
    case 'se': crop.width += dx; crop.height += dy; break
    case 'n': crop.y += dy; crop.height -= dy; break
    case 's': crop.height += dy; break
    case 'e': crop.width += dx; break
    case 'w': crop.x += dx; crop.width -= dx; break
  }

  // Enforce aspect ratio
  if (aspectRatio.value !== 'free') {
    const [w, h] = aspectRatio.value.split(':').map(Number)
    const ratio = w / h
    if (['nw', 'ne', 'sw', 'se', 'n', 's'].includes(handle)) {
      crop.width = crop.height * ratio
    } else {
      crop.height = crop.width / ratio
    }
  }

  // Clamp
  crop.width = Math.max(20, crop.width)
  crop.height = Math.max(20, crop.height)
}

function resetCrop() {
  if (imageRef.value) {
    const rect = imageRef.value.getBoundingClientRect()
    crop.x = rect.width * 0.1
    crop.y = rect.height * 0.1
    crop.width = rect.width * 0.8
    crop.height = rect.height * 0.8
  }
}

async function saveCrop() {
  if (!props.file || saving.value) return
  saving.value = true

  try {
    // Open save dialog
    const savePath = await save({
      defaultPath: `cropped_${props.file.name}`,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
    })

    if (!savePath) {
      saving.value = false
      return
    }

    const canvas = document.createElement('canvas')
    // Get natural image dimensions and calculate crop ratio
    const img = new Image()
    img.crossOrigin = 'anonymous'

    await new Promise((resolve, reject) => {
      img.onload = resolve
      img.onerror = reject
      img.src = imageUrl.value
    })

    // Calculate actual crop coordinates based on natural dimensions
    const displayWidth = imageRef.value.clientWidth
    const displayHeight = imageRef.value.clientHeight
    const scaleX = img.naturalWidth / displayWidth
    const scaleY = img.naturalHeight / displayHeight

    const cropX = crop.x * scaleX
    const cropY = crop.y * scaleY
    const cropW = crop.width * scaleX
    const cropH = crop.height * scaleY

    canvas.width = cropW
    canvas.height = cropH
    const ctx = canvas.getContext('2d')
    ctx.drawImage(img, cropX, cropY, cropW, cropH, 0, 0, cropW, cropH)

    // Convert to blob and save via Tauri plugin-fs
    const ext = savePath.split('.').pop().toLowerCase()
    const mimeType = ext === 'png' ? 'image/png' : 'image/jpeg'
    const blob = await new Promise(resolve => canvas.toBlob(resolve, mimeType, 0.95))
    const arrayBuffer = await blob.arrayBuffer()
    const uint8Array = new Uint8Array(arrayBuffer)

    await writeFile(savePath, uint8Array)

    emit('saved', savePath)
    emit('close')
  } catch (error) {
    console.error('Crop save failed:', error)
  } finally {
    saving.value = false
  }
}

function cancel() {
  emit('close')
}
</script>

<style scoped>
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
  transform: scale(1.2);
}
</style>