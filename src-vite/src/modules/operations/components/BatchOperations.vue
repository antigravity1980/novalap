<template>
  <div v-if="visible" class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-base-300 border border-base-200/50 rounded-box max-w-lg w-full overflow-hidden shadow-2xl flex flex-col max-h-[85vh]">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-base-200/40 flex items-center justify-between bg-base-200/30">
        <div class="flex items-center gap-2">
          <span class="text-xl">🛠️</span>
          <div>
            <h3 class="font-bold text-base-content">{{ $t('batch_ops.title') }}</h3>
            <p class="text-xs text-base-content/50">{{ $t('batch_ops.files_selected', { count: selectedFiles.length }) }}</p>
          </div>
        </div>
        <button class="btn btn-ghost btn-circle btn-sm hover:bg-base-100/50 text-base-content/70 hover:text-base-content" @click="close">✕</button>
      </div>

      <!-- Content -->
      <div class="p-5 flex-1 overflow-y-auto space-y-4">
        <!-- Tabs -->
        <div class="flex flex-wrap gap-2 p-1.5 bg-base-200/80 rounded-xl border border-base-200/50 shadow-inner">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="flex-1 min-w-[80px] text-[11px] font-bold py-2 px-3 rounded-lg transition-all duration-200 border border-transparent text-center select-none"
            :class="activeTab === tab.id 
              ? 'bg-primary text-primary-content shadow-md border-primary-focus/20 scale-105' 
              : 'bg-base-100/30 text-base-content/65 hover:text-base-content hover:bg-base-100/70 hover:shadow-sm'"
            @click="activeTab = tab.id"
          >
            {{ $t('batch_ops.tabs.' + tab.id) }}
          </button>
        </div>

        <!-- Tab contents -->
        <div class="bg-base-200/20 p-4 rounded-box border border-base-200/20 min-h-[220px]">
          <!-- 1. Resize -->
          <div v-if="activeTab === 'resize'" class="space-y-4">
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.fit_mode') }}</label>
                <select v-model="resize.fit" class="select select-bordered select-sm w-full">
                  <option value="contain">{{ $t('batch_ops.fit_options.contain') }}</option>
                  <option value="cover">{{ $t('batch_ops.fit_options.cover') }}</option>
                  <option value="fill">{{ $t('batch_ops.fit_options.fill') }}</option>
                  <option value="exact">{{ $t('batch_ops.fit_options.exact') }}</option>
                </select>
              </div>
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.preset_size') }}</label>
                <select @change="applyPreset" class="select select-bordered select-sm w-full">
                  <option value="">{{ $t('batch_ops.custom_size') }}</option>
                  <option value="512x512">512 × 512 (1:1)</option>
                  <option value="1024x1024">1024 × 1024 (1:1)</option>
                  <option value="1280x720">1280 × 720 (16:9)</option>
                  <option value="1920x1080">1920 × 1080 (16:9)</option>
                  <option v-for="p in customPresets" :key="p.name" :value="p.value">
                    {{ p.name }} ({{ p.value }})
                  </option>
                </select>
              </div>
            </div>

            <!-- Custom Presets Manager -->
            <div class="p-3 bg-base-100/40 rounded-lg border border-base-200/50 space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-base-content/70">Мои пресеты</span>
                <button 
                  @click="addCurrentAsPreset" 
                  class="btn btn-xs btn-primary rounded font-semibold"
                >
                  + Сохранить текущий
                </button>
              </div>
              <div v-if="customPresets.length === 0" class="text-[10px] text-base-content/40 italic">
                Нет сохраненных пресетов
              </div>
              <div v-else class="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto custom-scrollbar">
                <div 
                  v-for="(p, idx) in customPresets" 
                  :key="p.name" 
                  class="flex items-center gap-1 bg-base-200 px-2 py-1 rounded text-[10px] border border-neutral/15"
                >
                  <span class="font-medium cursor-pointer hover:text-primary" @click="selectCustomPreset(p.value)">{{ p.name }} ({{ p.value }})</span>
                  <button @click="deletePreset(idx)" class="text-error font-bold hover:scale-110 ml-0.5">✕</button>
                </div>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.width_px') }}</label>
                <input type="number" v-model.number="resize.width" class="input input-bordered input-sm w-full" min="10" />
              </div>
              <div>
                <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.height_px') }}</label>
                <input type="number" v-model.number="resize.height" class="input input-bordered input-sm w-full" min="10" />
              </div>
            </div>
          </div>

          <!-- 2. Convert -->
          <div v-if="activeTab === 'convert'" class="space-y-4">
            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.target_format') }}</label>
              <select v-model="convert.format" class="select select-bordered select-sm w-full">
                <option value="png">PNG ({{ $t('batch_ops.lossless_label') }})</option>
                <option value="jpeg">JPEG</option>
                <option value="webp">WebP</option>
                <option value="bmp">BMP</option>
                <option value="gif">GIF</option>
              </select>
            </div>

            <div v-if="['jpeg', 'webp'].includes(convert.format)" class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">{{ $t('batch_ops.quality') }}</span>
                <span class="font-mono text-primary font-bold">{{ convert.quality }}%</span>
              </div>
              <input type="range" min="10" max="100" v-model.number="convert.quality" class="range range-primary range-xs" />
            </div>
          </div>

          <!-- 3. Rename -->
          <div v-if="activeTab === 'rename'" class="space-y-4">
            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.new_name_mask') }}</label>
              <input
                type="text"
                v-model="rename.mask"
                class="input input-bordered input-sm w-full"
                placeholder="image_{counter}"
              />
              <p class="text-[10px] text-base-content/40 mt-1">
                {{ $t('batch_ops.name_mask_tip') }}
              </p>
            </div>

            <div>
              <label class="label text-xs text-base-content/60 font-semibold mb-1">{{ $t('batch_ops.start_counter_at') }}</label>
              <input type="number" v-model.number="rename.counterStart" class="input input-bordered input-sm w-full" min="1" />
            </div>
          </div>

          <!-- 4. Color Correct -->
          <div v-if="activeTab === 'color'" class="space-y-5">
            <div class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">{{ $t('batch_ops.saturation_multiplier') }}</span>
                <span class="font-mono text-primary font-bold">{{ color.saturation.toFixed(2) }}x</span>
              </div>
              <input type="range" min="0.0" max="3.0" step="0.1" v-model.number="color.saturation" class="range range-primary range-xs" />
              <div class="flex justify-between text-[10px] text-base-content/30">
                <span>{{ $t('batch_ops.grayscale') }}</span>
                <span>{{ $t('batch_ops.normal') }}</span>
                <span>{{ $t('batch_ops.vibrant') }}</span>
              </div>
            </div>

            <div class="space-y-1">
              <div class="flex justify-between text-xs">
                <span class="text-base-content/60 font-semibold">{{ $t('batch_ops.brightness_gamma') }}</span>
                <span class="font-mono text-primary font-bold">{{ color.gamma.toFixed(2) }}x</span>
              </div>
              <input type="range" min="0.1" max="3.0" step="0.1" v-model.number="color.gamma" class="range range-primary range-xs" />
              <div class="flex justify-between text-[10px] text-base-content/30">
                <span>{{ $t('batch_ops.darker') }}</span>
                <span>{{ $t('batch_ops.normal') }}</span>
                <span>{{ $t('batch_ops.brighter') }}</span>
              </div>
            </div>
          </div>

          <!-- 5. Compress -->
          <div v-if="activeTab === 'compress'" class="space-y-4">
            <div class="p-3 bg-base-300/50 rounded-lg flex items-center justify-between border border-base-200/30">
              <div>
                <h4 class="text-xs font-semibold text-base-content">{{ $t('batch_ops.lossy_png') }}</h4>
                <p class="text-[10px] text-base-content/50">{{ $t('batch_ops.lossy_png_hint') }}</p>
              </div>
              <span v-if="optimizers.pngquant === null" class="loading loading-spinner loading-xs"></span>
              <span v-else-if="optimizers.pngquant" class="badge badge-success badge-sm font-semibold">{{ $t('batch_ops.available') }}</span>
              <span v-else class="badge badge-neutral badge-sm font-semibold">{{ $t('batch_ops.not_installed') }}</span>
            </div>

            <div class="p-3 bg-base-300/50 rounded-lg flex items-center justify-between border border-base-200/30">
              <div>
                <h4 class="text-xs font-semibold text-base-content">{{ $t('batch_ops.lossless_jpeg') }}</h4>
                <p class="text-[10px] text-base-content/50">{{ $t('batch_ops.lossless_jpeg_hint') }}</p>
              </div>
              <span v-if="optimizers.cjpeg === null" class="loading loading-spinner loading-xs"></span>
              <span v-else-if="optimizers.cjpeg" class="badge badge-success badge-sm font-semibold">{{ $t('batch_ops.available') }}</span>
              <span v-else class="badge badge-neutral badge-sm font-semibold">{{ $t('batch_ops.not_installed') }}</span>
            </div>

            <div class="alert alert-info py-2 px-3 text-xs leading-normal">
              <span>{{ $t('batch_ops.compress_notice') }}</span>
            </div>
          </div>

          <!-- 6. Strip -->
          <div v-if="activeTab === 'strip'" class="space-y-4">
            <div class="alert alert-warning py-3 px-4 text-xs leading-normal flex items-start gap-2 bg-warning/10 border-warning/20 text-warning">
              <span>⚠️</span>
              <div>
                <h4 class="font-bold">{{ $t('batch_ops.anonymize_title') }}</h4>
                <p class="mt-1 text-[11px] opacity-90">
                  {{ $t('batch_ops.anonymize_desc') }}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Progress Overlay -->
      <div v-if="processing" class="absolute inset-0 bg-base-300/90 backdrop-blur-sm z-30 flex flex-col items-center justify-center p-6 text-center">
        <span class="loading loading-spinner loading-lg text-primary mb-4"></span>
        <h4 class="font-bold text-base-content">{{ $t('batch_ops.processing_files') }}</h4>
        <p class="text-xs text-base-content/50 mt-1">
          {{ $t('batch_ops.processing_hint', { tab: $t('batch_ops.tabs.' + activeTab), count: selectedFiles.length }) }}
        </p>
      </div>

      <!-- Result message -->
      <div v-if="resultMessage" class="absolute inset-0 bg-base-300/95 z-30 flex flex-col items-center justify-center p-6 text-center">
        <div class="w-12 h-12 rounded-full bg-success/10 flex items-center justify-center text-success text-2xl mb-3">✓</div>
        <h4 class="font-bold text-base-content text-lg">{{ $t('batch_ops.batch_complete') }}</h4>
        <p class="text-sm text-base-content/70 mt-1 max-w-sm">{{ resultMessage }}</p>
        <button class="btn btn-primary btn-sm mt-6 px-6" @click="closeResult">{{ $t('batch_ops.done') }}</button>
      </div>

      <!-- Footer -->
      <div class="px-5 py-4 border-t border-base-200/40 flex items-center justify-between bg-base-200/10">
        <button class="btn btn-ghost btn-sm hover:bg-base-100/50" @click="close">{{ $t('batch_ops.cancel') }}</button>
        <button
          class="btn btn-primary btn-sm px-6"
          :disabled="selectedFiles.length === 0 || processing || isCompressUnavailable"
          @click="apply"
        >
          {{ $t('batch_ops.apply_changes') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  visible: { type: Boolean, default: false },
  selectedFiles: { type: Array, default: () => [] },
})

const emit = defineEmits(['close', 'success'])

const { t } = useI18n()

const activeTab = ref('resize')
const processing = ref(false)
const resultMessage = ref('')

const customPresets = ref([])

function loadCustomPresets() {
  const stored = localStorage.getItem('batch_size_presets')
  if (stored) {
    try {
      customPresets.value = JSON.parse(stored)
    } catch (e) {
      console.error(e)
    }
  } else {
    customPresets.value = []
  }
}

function saveCustomPresets() {
  localStorage.setItem('batch_size_presets', JSON.stringify(customPresets.value))
}

function addCurrentAsPreset() {
  const w = resize.width
  const h = resize.height
  if (!w || !h) return
  const name = prompt('Введите название пресета:', `${w}x${h}`)
  if (!name) return
  customPresets.value.push({
    name,
    value: `${w}x${h}`
  })
  saveCustomPresets()
}

function selectCustomPreset(val) {
  const [w, h] = val.split('x').map(Number)
  resize.width = w
  resize.height = h
}

function deletePreset(idx) {
  customPresets.value.splice(idx, 1)
  saveCustomPresets()
}

watch(() => props.visible, (val) => {
  if (val) {
    loadCustomPresets()
  }
}, { immediate: true })

const tabs = [
  { id: 'resize' },
  { id: 'convert' },
  { id: 'rename' },
  { id: 'color' },
  { id: 'compress' },
  { id: 'strip' },
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
      resultMessage.value = t('batch_ops.success_message', { succeeded: result.succeeded, total: result.total })
      if (result.failed > 0) {
        resultMessage.value += t('batch_ops.failed_message', { failed: result.failed })
      }
      emit('success')
    }
  } catch (error) {
    console.error('Batch processing failed:', error)
    alert(typeof error === 'string' ? error : (t('batch_ops.title') + ' failed'))
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
