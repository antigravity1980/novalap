<template>
  <div class="prompt-viewer bg-base-200/50 rounded-lg p-3 max-w-md w-full">
    <div class="flex items-center justify-between mb-2">
      <h3 class="text-sm font-semibold">{{ $t('viewer.prompt_inspector') }}</h3>
      <button class="btn btn-ghost btn-xs" @click="loadMetadata" :disabled="loading">
        {{ loading ? $t('viewer.loading') : '🔄' }}
      </button>
    </div>

    <div v-if="loading" class="text-center py-4">
      <span class="loading loading-spinner loading-sm text-primary"></span>
    </div>

    <div v-else-if="error" class="text-error text-xs p-2">
      {{ error }}
    </div>

    <div v-else-if="!metadata" class="text-base-content/40 text-xs text-center py-4">
      {{ $t('viewer.select_file_metadata') }}
    </div>

    <div v-else class="metadata-content space-y-2 max-h-96 overflow-y-auto text-xs">
      <!-- Engine badge -->
      <div v-if="metadata.source_engine" class="badge badge-primary badge-sm">
        {{ metadata.source_engine }}
      </div>

      <!-- Positive Prompt -->
      <div v-if="metadata.positive_prompt" class="metadata-section">
        <div class="font-semibold text-green-600 dark:text-green-400">{{ $t('viewer.positive_prompt') }}</div>
        <div class="mt-0.5 text-base-content/80 bg-base-100 rounded p-1.5 break-words">
          {{ metadata.positive_prompt }}
        </div>
      </div>

      <!-- Negative Prompt -->
      <div v-if="metadata.negative_prompt" class="metadata-section">
        <div class="font-semibold text-red-600 dark:text-red-400">{{ $t('viewer.negative_prompt') }}</div>
        <div class="mt-0.5 text-base-content/80 bg-base-100 rounded p-1.5 break-words">
          {{ metadata.negative_prompt }}
        </div>
      </div>

      <!-- Generation params -->
      <div class="grid grid-cols-2 gap-1">
        <div v-if="metadata.seed != null" class="param-item">
          <span class="text-base-content/50">Seed:</span>
          <span class="font-mono">{{ metadata.seed }}</span>
        </div>
        <div v-if="metadata.steps != null" class="param-item">
          <span class="text-base-content/50">Steps:</span>
          <span>{{ metadata.steps }}</span>
        </div>
        <div v-if="metadata.cfg_scale != null" class="param-item">
          <span class="text-base-content/50">CFG:</span>
          <span>{{ metadata.cfg_scale }}</span>
        </div>
        <div v-if="metadata.model" class="param-item col-span-2">
          <span class="text-base-content/50">Model:</span>
          <span class="font-medium">{{ metadata.model }}</span>
        </div>
      </div>

      <!-- LoRAs -->
      <div v-if="metadata.loras && metadata.loras.length > 0">
        <div class="font-semibold text-base-content/70">LoRAs ({{ metadata.loras.length }})</div>
        <div class="flex flex-wrap gap-1 mt-0.5">
          <span v-for="lora in metadata.loras" :key="lora" class="badge badge-ghost badge-xs">
            {{ lora }}
          </span>
        </div>
      </div>

      <!-- Workflow (collapsible) -->
      <div v-if="metadata.workflow">
        <details class="metadata-section">
          <summary class="font-semibold text-base-content/70 cursor-pointer">{{ $t('viewer.workflow_json') }}</summary>
          <pre class="mt-0.5 text-[10px] bg-base-100 rounded p-1.5 overflow-x-auto max-h-32">{{ formatWorkflow(metadata.workflow) }}</pre>
        </details>
      </div>

      <!-- Raw metadata -->
      <div v-if="metadata.raw_metadata && metadata.raw_metadata.length > 0">
        <details class="metadata-section">
          <summary class="font-semibold text-base-content/50 cursor-pointer">
            {{ $t('viewer.raw_metadata', { count: metadata.raw_metadata.length }) }}
          </summary>
          <div class="mt-0.5 space-y-0.5">
            <div v-for="entry in metadata.raw_metadata" :key="entry.key" class="text-[10px] bg-base-100 rounded p-1">
              <span class="font-medium">{{ entry.key }}:</span>
              <span class="text-base-content/70 break-all">{{ entry.value }}</span>
            </div>
          </div>
        </details>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  filePath: { type: String, default: '' },
})

const metadata = ref(null)
const loading = ref(false)
const error = ref(null)

watch(() => props.filePath, (newPath) => {
  if (newPath) loadMetadata()
})

async function loadMetadata() {
  if (!props.filePath) return

  loading.value = true
  error.value = null
  metadata.value = null

  try {
    metadata.value = await invoke('parse_ai_metadata', { path: props.filePath })
  } catch (err) {
    error.value = typeof err === 'string' ? err : (t('viewer.loading') === 'Загрузка...' ? 'Не удалось загрузить метаданные' : 'Failed to load metadata')
  } finally {
    loading.value = false
  }
}

function formatWorkflow(workflow) {
  if (!workflow) return ''
  try {
    return JSON.stringify(JSON.parse(workflow), null, 2).slice(0, 2000)
  } catch {
    return workflow.slice(0, 2000)
  }
}

defineExpose({ loadMetadata })
</script>

<style scoped>
.metadata-section {
  margin-top: 0;
}
.param-item {
  display: flex;
  gap: 4px;
  align-items: center;
}
</style>