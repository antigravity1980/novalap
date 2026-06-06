<template>
  <div class="tree-folder-node select-none">
    <div
      class="tree-item flex items-center gap-1.5 px-2.5 py-1.5 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150"
      :class="{
        'bg-primary/10 text-primary font-bold shadow-sm': isActive,
        'text-base-content/70 hover:bg-base-100/30 hover:text-base-content': !isActive,
      }"
      @click="navigate"
    >
      <!-- Expand chevron arrow -->
      <span
        class="chevron flex items-center justify-center text-[10px] w-4 h-4 rounded-md hover:bg-base-100/40 text-base-content/40 hover:text-base-content/80 transition-transform duration-200"
        :class="{ 'rotate-90': isExpanded }"
        @click.stop="toggle"
      >
        <span v-if="hasChildren">▸</span>
        <span v-else>&nbsp;</span>
      </span>

      <!-- Folder icon -->
      <span class="text-sm shrink-0">
        {{ isExpanded ? '📂' : '📁' }}
      </span>

      <span class="truncate flex-1">{{ folder.name }}</span>
    </div>

    <!-- Children list -->
    <div v-if="isExpanded && children.length" class="tree-children ml-4 border-l border-base-content/5 pl-2 mt-0.5 space-y-0.5">
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
    await navigationStore.expandTreeFolder(props.folder.path)
    children.value = navigationStore.treeFolders[props.folder.path] || []
  }
}

function navigate() {
  emit('navigate', props.folder.path)
}
</script>

<style scoped>
.chevron {
  font-family: monospace;
}
.tree-item {
  margin-bottom: 2px;
}
</style>