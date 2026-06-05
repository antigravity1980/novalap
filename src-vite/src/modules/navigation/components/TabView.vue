<template>
  <div class="tab-view flex flex-col h-full">
    <!-- Tabs bar -->
    <div class="tabs-bar flex items-center bg-base-200/50 border-b border-base-200 overflow-x-auto">
      <div
        v-for="(tab, index) in tabs"
        :key="tab.id"
        class="tab-item flex items-center gap-1 px-3 py-1.5 cursor-pointer text-sm border-r border-base-200 whitespace-nowrap"
        :class="{ 'bg-base-100 font-medium': activeTabId === tab.id }"
        @click="switchTab(tab.id)"
        draggable="true"
        @dragstart="onDragStart($event, tab.id)"
        @dragover.prevent="onDragOver($event, tab.id)"
        @drop="onDrop($event, tab.id)"
      >
        <span class="truncate max-w-[100px]">{{ tab.label }}</span>
        <button
          v-if="tabs.length > 1"
          class="close-btn text-base-content/30 hover:text-error text-xs ml-1"
          @click.stop="closeTab(tab.id)"
        >
          ✕
        </button>
      </div>
      <button class="add-tab px-2 py-1.5 text-base-content/50 hover:text-base-content flex-shrink-0" @click="addTab">
        +
      </button>
    </div>

    <!-- Tab content -->
    <div class="tab-content flex-1 overflow-hidden relative">
      <div v-for="tab in tabs" :key="tab.id" v-show="tab.id === activeTabId" class="tab-pane absolute inset-0">
        <slot name="tab-content" :tab="tab" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  initialTabs: {
    type: Array,
    default: () => [{ id: 'tab-1', label: 'Explorer 1', path: '' }]
  }
})

const emit = defineEmits(['switchTab', 'addTab', 'closeTab'])

const tabs = ref([...props.initialTabs])
const activeTabId = ref(tabs.value[0]?.id || 'tab-1')
let tabCounter = 1

const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))

function addTab(label) {
  tabCounter++
  const newTab = {
    id: `tab-${Date.now()}`,
    label: label || `Tab ${tabCounter}`,
    path: ''
  }
  tabs.value.push(newTab)
  activeTabId.value = newTab.id
  emit('addTab', newTab)
  return newTab
}

function switchTab(tabId) {
  activeTabId.value = tabId
  const tab = tabs.value.find(t => t.id === tabId)
  if (tab) emit('switchTab', tab)
}

function closeTab(tabId) {
  if (tabs.value.length <= 1) return
  const index = tabs.value.findIndex(t => t.id === tabId)
  if (index >= 0) {
    tabs.value.splice(index, 1)
    if (activeTabId.value === tabId) {
      const newIndex = Math.min(index, tabs.value.length - 1)
      if (newIndex >= 0) {
        activeTabId.value = tabs.value[newIndex].id
        emit('switchTab', tabs.value[newIndex])
      }
    }
  }
  emit('closeTab', tabId)
}

function updateTabLabel(tabId, label) {
  const tab = tabs.value.find(t => t.id === tabId)
  if (tab) tab.label = label
}

// Drag & Drop reorder
let draggedTabId = null

function onDragStart(event, tabId) {
  draggedTabId = tabId
  event.dataTransfer.effectAllowed = 'move'
}

function onDragOver(event, tabId) {
  event.dataTransfer.dropEffect = 'move'
}

function onDrop(event, tabId) {
  if (draggedTabId && draggedTabId !== tabId) {
    const fromIndex = tabs.value.findIndex(t => t.id === draggedTabId)
    const toIndex = tabs.value.findIndex(t => t.id === tabId)
    if (fromIndex >= 0 && toIndex >= 0) {
      const [movedTab] = tabs.value.splice(fromIndex, 1)
      tabs.value.splice(toIndex, 0, movedTab)
    }
  }
  draggedTabId = null
}

defineExpose({ tabs, activeTabId, activeTab, addTab, closeTab, updateTabLabel, switchTab })
</script>

<style scoped>
.tabs-bar {
  scrollbar-width: thin;
}
.tab-item {
  user-select: none;
  max-width: 160px;
  transition: background 0.1s;
}
.tab-item:hover {
  background: var(--fallback-b2, oklch(var(--b2)));
}
.close-btn {
  visibility: hidden;
}
.tab-item:hover .close-btn {
  visibility: visible;
}
.tab-pane {
  overflow: auto;
}
</style>