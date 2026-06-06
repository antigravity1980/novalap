<template>
  <div class="explorer-tree py-2">
    <div class="tree-content space-y-1">
      <!-- Drives list -->
      <div v-for="drive in navigationStore.drives" :key="drive.path" class="tree-node px-1">
        <div
          class="tree-item flex items-center justify-between px-3 py-2 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150 relative"
          :class="{
            'bg-primary/15 text-primary font-bold active-nav-item': drive.path === navigationStore.currentPath,
            'text-base-content/75 hover:bg-base-100/35 hover:text-base-content': drive.path !== navigationStore.currentPath,
          }"
          @click="navigateTo(drive.path)"
        >
          <div class="flex items-center gap-2 truncate">
            <span class="text-sm">💾</span>
            <span class="truncate font-semibold">{{ drive.name }}</span>
          </div>
          <span v-if="drive.is_removable" class="text-[9px] uppercase tracking-wider bg-base-300 text-base-content/50 px-1 py-0.5 rounded shrink-0">
            {{ $t('explorer.drives.removable') }}
          </span>
        </div>

        <!-- Drive root child folders -->
        <div v-if="expandedNodes[drive.path]" class="tree-children ml-2 mt-1 border-l border-base-content/5 pl-2 space-y-0.5">
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
import { reactive, onMounted, watch } from 'vue'
import { useNavigationStore } from '../store'
import { useGalleryStore } from '../../gallery/store'
import TreeFolderNode from './TreeFolderNode.vue'

const navigationStore = useNavigationStore()
const galleryStore = useGalleryStore()
const expandedNodes = reactive({})

async function navigateTo(path) {
  if (path) {
    expandedNodes[path] = true
    await navigationStore.expandTreeFolder(path)
    await navigationStore.navigateTo(path)
    galleryStore.setFiles(navigationStore.folders)
  }
}

async function expandFolder(path) {
  expandedNodes[path] = true
  await navigationStore.expandTreeFolder(path)
}

onMounted(() => {
  // If there's an active path, expand its drive node
  if (navigationStore.currentPath) {
    const drive = navigationStore.drives.find(d => navigationStore.currentPath.startsWith(d.path))
    if (drive) {
      expandedNodes[drive.path] = true
    }
  }
})

// Авторазвёртывание дерева при навигации
watch(() => navigationStore.currentPath, (newPath) => {
  if (newPath) {
    const drive = navigationStore.drives.find(d => newPath.startsWith(d.path))
    if (drive) {
      expandedNodes[drive.path] = true
    }
  }
})
</script>

<style scoped>
.explorer-tree {
  height: 100%;
}
</style>