<template>
  <div class="explorer-tree">
    <div class="tree-header">
      <h3 class="text-sm font-semibold px-2 py-1">Explorer</h3>
    </div>
    <div class="tree-content">
      <!-- Диски -->
      <div v-for="drive in navigationStore.drives" :key="drive.path" class="tree-node">
        <div
          class="tree-item flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-base-200 rounded"
          :class="{ 'bg-primary/10': drive.path === navigationStore.currentPath }"
          @click="navigateTo(drive.path)"
        >
          <span class="text-lg">💾</span>
          <span class="text-sm truncate">{{ drive.name }}</span>
          <span v-if="drive.is_removable" class="text-xs text-base-content/50">(removable)</span>
        </div>

        <!-- Поддерево папок для активного диска -->
        <div v-if="expandedNodes[drive.path]" class="tree-children ml-3">
          <TreeFolderNode
            v-for="folder in navigationStore.treeFolders[drive.path]"
            :key="folder.path"
            :folder="folder"
            @navigate="navigateTo"
            @expand="expandFolder"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'
import { useNavigationStore } from '../store'
import TreeFolderNode from './TreeFolderNode.vue'

const navigationStore = useNavigationStore()
const expandedNodes = reactive({})

async function navigateTo(path) {
  if (path) {
    expandedNodes[path] = true
    await navigationStore.expandTreeFolder(path)
    await navigationStore.navigateTo(path)
  }
}

async function expandFolder(path) {
  expandedNodes[path] = true
  await navigationStore.expandTreeFolder(path)
}

async function toggleExpand(path) {
  if (expandedNodes[path]) {
    expandedNodes[path] = false
  } else {
    await expandFolder(path)
  }
}
</script>

<style scoped>
.explorer-tree {
  height: 100%;
  overflow-y: auto;
}
.tree-content {
  min-height: 100px;
}
</style>