<template>
  <div v-if="visible" class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-base-300 border border-base-200/50 rounded-box max-w-lg w-full overflow-hidden shadow-2xl flex flex-col max-h-[85vh]">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-base-200/40 flex items-center justify-between bg-base-200/30">
        <div class="flex items-center gap-2">
          <span class="text-xl">🛠️</span>
          <div>
            <h3 class="font-bold text-base-content">Batch Operations</h3>
            <p class="text-xs text-base-content/50">{{ selectedFiles.length }} files selected</p>
          </div>
        </div>
        <button class="btn btn-ghost btn-circle btn-sm hover:bg-base-100/50 text-base-content/70 hover:text-base-content" @click="close">✕</button>
      </div>

      <!-- Content -->
      <div class="p-5 flex-1 overflow-y-auto space-y-4">
        <!-- Tabs -->
        <div class="tabs tabs-boxed bg-base-200/40 p-1 flex flex-wrap gap-1 rounded-lg">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="tab tab-sm flex-1 text-xs py-1.5 transition-all duration-200 font-medium"
            :class="{ 'tab-active bg-primary text-primary-content rounded-md': activeTab === tab.id }"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>

        <!-- Tab contents -->
        <div class="bg-base-200/20 p-4 rounded-box border border-base-200/20 min-h-[220px]">
          <!-- 1. Resize -->
          <div v-if="activeTab === 'resize'" class="space-y-4">
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">Fit Mode</label>
                <select v-model="resize.fit" class="select select-bordered select-sm w-full">
                  <option value="contain">Contain (Aspect Ratio)</option>
                  <option value="cover">Cover (Crop)</option>
                  <option value="fill">Fill (Stretch)</option>
                  <option value="exact">Exact (No aspect check)</option>
                </select>
              </div>
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">Preset Size</label>
                <select @change="applyPreset" class="select select-bordered select-sm w-full">
                  <option value="">Custom Size</option>
                  <option value="512x512">512 × 512 (1:1)</option>
                  <option value="1024x1024">1024 × 1024 (1:1)</option>
                  <option value="1280x720">1280 × 720 (16:9)</option>
                  <option value="1920x1080">1920 × 1080 (16:9)</option>
                </select>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">Width (px)</label>
                <input type="number" v-model.number="resize.width" class="input input-bordered input-sm w-full" min="10" />
              </div>
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">Height (px)</label>
                <input type="number" v-model.number="resize.height" class="input input-bordered input-sm w-full" min="10" />
              </div>
            </div>
          </div>

          <!-- 2. Convert -->
          <div v-if="activeTab === 'convert'" class="space-y-4">
            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">Target Format</label>
              <select v-model="convert.format" class="select select-bordered select-sm w-full">
                <option value="png">PNG (Lossless)</option>
                <option value="jpeg">JPEG</option>
                <option value="webp">WebP</option>
                <option value="bmp">BMP</option>
                <option value="gif">GIF</option>
              </select>
            </div>

            <div v-if="['jpeg', 'webp'].includes(convert.format)" class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">Quality</span>
                <span class="font-mono text-primary font-bold">{{ convert.quality }}%</span>
              </div>
              <input type="range" min="10" max="100" v-model.number="convert.quality" class="range range-primary range-xs" />
            </div>
          </div>

          <!-- 3. Rename -->
          <div v-if="activeTab === 'rename'" class="space-y-4">
            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">New Name Mask</label>
              <input
                type="text"
                v-model="rename.mask"
                class="input input-bordered input-sm w-full"
                placeholder="e.g. vacation_{counter}"
              />
              <p class="text-[10px] text-base-content/40 mt-1">
                Tip: Use <code class="font-mono bg-base-300 px-1 py-0.5 rounded text-primary">{counter}</code> for a number or <code class="font-mono bg-base-300 px-1 py-0.5 rounded text-primary">{ext}</code> to preserve file extension.
              </p>
            </div>

            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">Start Counter At</label>
              <input type="number" v-model.number="rename.counterStart" class="input input-bordered input-sm w-full" min="1" />
            </div>
          </div>

          <!-- 4. Color Correct -->
          <div v-if="activeTab === 'color'" class="space-y-5">
            <div class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">Saturation multiplier</span>
                <span class="font-mono text-primary font-bold">{{ color.saturation.toFixed(2) }}x</span>
              </div>
              <input type="range" min="0.0" max="3.0" step="0.1" v-model.number="color.saturation" class="range range-primary range-xs" />
              <div class="flex justify-between text-[10px] text-base-content/30">
                <span>Grayscale (0)</span>
                <span>Normal (1)</span>
                <span>Vibrant (3)</span>
              </div>
            </div>

            <div class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">Brightness / Gamma</span>
                <span class="font-mono text-primary font-bold">{{ color.gamma.toFixed(2) }}x</span>
              </div>
              <input type="range" min="0.1" max="3.0" step="0.1" v-model.number="color.gamma" class="range range-primary range-xs" />
              <div class="flex justify-between text-[10px] text-base-content/30">
                <span>Darker (0.1)</span>
                <span>Normal (1)</span>
                <span>Brighter (3)</span>
              </div>
            </div>
          </div>

          <!-- 5. Compress -->
          <div v-if="activeTab === 'compress'" class="space-y-4">
            <div class="p-3 bg-base-300/50 rounded-lg flex items-center justify-between border border-base-200/30">
              <div>
                <h4 class="text-xs font-semibold text-base-content">Lossy PNG Compression</h4>
                <p class="text-[10px] text-base-content/50">Uses pngquant library to dramatically decrease PNG size.</p>
              </div>
              <span v-if="optimizers.pngquant === null" class="loading loading-spinner loading-xs"></span>
              <span v-else-if="optimizers.pngquant" class="badge badge-success badge-sm font-semibold">Available</span>
              <span v-else class="badge badge-neutral badge-sm font-semibold">Not Installed</span>
            </div>

            <div class="p-3 bg-base-300/50 rounded-lg flex items-center justify-between border border-base-200/30">
              <div>
                <h4 class="text-xs font-semibold text-base-content">Lossless JPEG Compression</h4>
                <p class="text-[10px] text-base-content/50">Uses mozjpeg compiler to perform near-lossless JPEG shrinkage.</p>
              </div>
              <span v-if="optimizers.cjpeg === null" class="loading loading-spinner loading-xs"></span>
              <span v-else-if="optimizers.cjpeg" class="badge badge-success badge-sm font-semibold">Available</span>
              <span v-else class="badge badge-neutral badge-sm font-semibold">Not Installed</span>
            </div>

            <div class="alert alert-info py-2 px-3 text-xs leading-normal">
              <span>Optimizing relies on local command-line tools `pngquant` and `cjpeg` (mozjpeg). If unavailable, please install them via your package manager.</span>
            </div>
          </div>

          <!-- 6. Strip -->
          <div v-if="activeTab === 'strip'" class="space-y-4">
            <div class="alert alert-warning py-3 px-4 text-xs leading-normal flex items-start gap-2 bg-warning/10 border-warning/20 text-warning">
              <span>⚠️</span>
              <div>
                <h4 class="font-bold">Anonymize Content & Remove Metadata</h4>
                <p class="mt-1 text-[11px] opacity-90">
                  This operation strips all embedded AI tags, generation parameters, ComfyUI workflows, negative prompts, and seeds. The visual image data is kept intact, but size is reduced. This operation is **irreversible** unless you copy files first.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Progress Overlay -->
      <div v-if="processing" class="absolute inset-0 bg-base-300/90 backdrop-blur-sm z-30 flex flex-col items-center justify-center p-6 text-center">
        <span class="loading loading-spinner loading-lg text-primary mb-4"></span>
        <h4 class="font-bold text-base-content">Processing files...</h4>
        <p class="text-xs text-base-content/50 mt-1">Applying {{ activeTab }} to {{ selectedFiles.length }} items.</p>
      </div>

      <!-- Result message -->
      <div v-if="resultMessage" class="absolute inset-0 bg-base-300/95 z-30 flex flex-col items-center justify-center p-6 text-center">
        <div class="w-12 h-12 rounded-full bg-success/10 flex items-center justify-center text-success text-2xl mb-3">✓</div>
        <h4 class="font-bold text-base-content text-lg">Batch Job Complete</h4>
        <p class="text-sm text-base-content/70 mt-1 max-w-sm">{{ resultMessage }}</p>
        <button class="btn btn-primary btn-sm mt-6 px-6" @click="closeResult">Done</button>
      </div>

      <!-- Footer -->
      <div class="px-5 py-4 border-t border-base-200/40 flex items-center justify-between bg-base-200/10">
        <button class="btn btn-ghost btn-sm hover:bg-base-100/50" @click="close">Cancel</button>
        <button
          class="btn btn-primary btn-sm px-6"
          :disabled="selectedFiles.length === 0 || processing || isCompressUnavailable"
          @click="apply"
        >
          Apply Changes
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  visible: { type: Boolean, default: false },
  selectedFiles: { type: Array, default: () => [] },
})

const emit = defineEmits(['close', 'success'])

const activeTab = ref('resize')
const processing = ref(false)
const resultMessage = ref('')

const tabs = [
  { id: 'resize', label: 'Resize' },
  { id: 'convert', label: 'Convert' },
  { id: 'rename', label: 'Rename' },
  { id: 'color', label: 'Color Correct' },
  { id: 'compress', label: 'Compress' },
  { id: 'strip', label: 'Strip Meta' },
]

// State for each task
const resize = reactive({
  width: 1024,
  height: 1024,
  fit: 'contain',
})

const convert = reactive({
  format: 'png',
  quality: 90,
})

const rename = reactive({
  mask: 'image_{counter}',
  counterStart: 1,
})

const color = reactive({
  saturation: 1.0,
  gamma: 1.0,
})

const optimizers = reactive({
  pngquant: null,
  cjpeg: null,
})

// Check external optimizer binaries when tab is selected
watch(activeTab, async (tab) => {
  if (tab === 'compress') {
    optimizers.pngquant = null
    optimizers.cjpeg = null
    try {
      optimizers.pngquant = await invoke('check_optimizer', { name: 'pngquant' })
    } catch {
      optimizers.pngquant = false
    }
    try {
      optimizers.cjpeg = await invoke('check_optimizer', { name: 'cjpeg' })
    } catch {
      optimizers.cjpeg = false
    }
  }
})

const isCompressUnavailable = computed(() => {
  if (activeTab.value !== 'compress') return false
  return !optimizers.pngquant && !optimizers.cjpeg
})

function applyPreset(event) {
  const val = event.target.value
  if (!val) return
  const [w, h] = val.split('x').map(Number)
  resize.width = w
  resize.height = h
}

async function apply() {
  if (props.selectedFiles.length === 0) return
  processing.value = true
  resultMessage.value = ''

  try {
    let result = null

    // Backup originals for Safety Undo
    await invoke('backup_originals', { paths: props.selectedFiles }).catch(() => {})

    switch (activeTab.value) {
      case 'resize':
        result = await invoke('batch_resize', {
          files: props.selectedFiles,
          preset: {
            width: resize.width,
            height: resize.height,
            fit: resize.fit,
          },
        })
        break
      case 'convert':
        result = await invoke('batch_convert', {
          files: props.selectedFiles,
          targetFormat: convert.format,
          quality: convert.quality,
        })
        break
      case 'rename':
        result = await invoke('batch_rename', {
          files: props.selectedFiles,
          mask: rename.mask,
          counterStart: rename.counterStart,
        })
        break
      case 'color':
        result = await invoke('batch_color_correct', {
          files: props.selectedFiles,
          saturation: color.saturation,
          gamma: color.gamma,
        })
        break
      case 'compress':
        // Compress based on formats
        const pngFiles = props.selectedFiles.filter((f) => f.toLowerCase().endsWith('.png'))
        const jpgFiles = props.selectedFiles.filter((f) =>
          ['.jpg', '.jpeg'].some((ext) => f.toLowerCase().endsWith(ext))
        )

        let succ = 0
        let errs = []

        if (pngFiles.length > 0 && optimizers.pngquant) {
          const res = await invoke('optimize_with_pngquant', { files: pngFiles })
          succ += res.succeeded
          errs = [...errs, ...res.errors]
        }
        if (jpgFiles.length > 0 && optimizers.cjpeg) {
          const res = await invoke('optimize_with_mozjpeg', { files: jpgFiles })
          succ += res.succeeded
          errs = [...errs, ...res.errors]
        }

        result = {
          total: pngFiles.length + jpgFiles.length,
          succeeded: succ,
          failed: pngFiles.length + jpgFiles.length - succ,
          errors: errs,
        }
        break
      case 'strip':
        result = await invoke('strip_metadata', { files: props.selectedFiles })
        break
    }

    if (result) {
      resultMessage.value = `Successfully processed ${result.succeeded} of ${result.total} files.`
      if (result.failed > 0) {
        resultMessage.value += ` Failed: ${result.failed}.`
      }
      emit('success')
    }
  } catch (error) {
    console.error('Batch processing failed:', error)
    alert(typeof error === 'string' ? error : 'Operation failed')
  } finally {
    processing.value = false
  }
}

function close() {
  emit('close')
}

function closeResult() {
  resultMessage.value = ''
  emit('close')
}
</script>

<style scoped>
/* Scoped modifications */
.tab-active {
  color: var(--fallback-pc, oklch(var(--pc)));
}
</style>
