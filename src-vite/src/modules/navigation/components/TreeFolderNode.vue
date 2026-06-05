<template>
  <div class="tree-folder-node">
    <div
      class="tree-item flex items-center gap-1 px-2 py-0.5 cursor-pointer hover:bg-base-200 rounded text-sm"
      :class="{ 'bg-primary/10 text-primary': isActive }"
      @click="navigate"
    >
      <span class="chevron text-xs w-4" @click.stop="toggle">
        {{ hasChildren ? (isExpanded ? '▼' : '▶') : '&nbsp;' }}
      </span>
      <span class="text-base">📁</span>
      <span class="truncate">{{ folder.name }}</span>
    </div>

    <div v-if="isExpanded && children.length" class="tree-children ml-3">
      <TreeFolderNode
        v-for="child in children"
        :key="child.path"
        :folder="child"
        :active-path="activePath"
        @navigate="(p) => $emit('navigate', p)"
        @expand="(p) => $emit('expand', p)"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useNavigationStore } from '../store'

const props = defineProps({
  folder: { type: Object, required: true },
  activePath: { type: String, default: '' },
})

const emit = defineEmits(['navigate', 'expand'])

const navigationStore = useNavigationStore()
const isExpanded = ref(false)
const children = ref(props.folder.children || [])
const hasChildren = computed(() => props.folder.has_subfolders)
const isActive = computed(() => props.folder.path === navigationStore.currentPath)

watch(() => props.folder.has_subfolders, (val) => {
  if (!val) isExpanded.value = false
})

async function toggle() {
  if (!hasChildren.value) return
  isExpanded.value = !isExpanded.value
  if (isExpanded.value && (!children.value || children.value.length === 0)) {
    emit('expand', props.folder.path)
    const treeData = await navigationStore.expandTreeFolder(props.folder.path)
    children.value = navigationStore.treeFolders[props.folder.path] || []
  }
}

function navigate() {
  emit('navigate', props.folder.path)
}
</script>