<template>
  <div class="explorer-tree py-2">
    <div class="tree-content space-y-1">
      <!-- Drives list -->
      <div v-for="drive in visibleDrives" :key="drive.path" class="tree-node px-1">
        <div
          class="tree-item flex items-center justify-between px-3 py-2 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150 relative"
          :class="{
            'bg-primary/15 text-primary font-bold active-nav-item': drive.path === navigationStore.currentPath && dragOverDrivePath !== drive.path,
            'text-base-content/75 hover:bg-base-100/35 hover:text-base-content': drive.path !== navigationStore.currentPath && dragOverDrivePath !== drive.path,
            'bg-secondary/20 text-secondary border border-dashed border-secondary/50': dragOverDrivePath === drive.path,
          }"
          @click="navigateTo(drive.path)"
          @dragover.prevent="dragOverDrivePath = drive.path"
          @dragenter.prevent="dragOverDrivePath = drive.path"
          @dragleave="dragOverDrivePath = ''"
          @drop.prevent="handleDriveDrop($event, drive.path)"
        >
          <div class="flex items-center gap-1.5 truncate">
            <!-- Expand chevron arrow for drive -->
            <span
              class="chevron flex items-center justify-center text-[10px] w-4 h-4 rounded-md hover:bg-base-100/40 text-base-content/40 hover:text-base-content/80 transition-transform duration-200"
              :class="{ 'rotate-90': expandedNodes[drive.path] }"
              @click.stop="toggleDrive(drive.path)"
            >
              ▸
            </span>
            <IconDrive class="w-4 h-4 shrink-0 text-base-content/70" />
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
import { reactive, onMounted, watch, computed, ref } from 'vue'
import { useNavigationStore } from '../store'
import { useGalleryStore } from '../../gallery/store'
import { useConfigStore } from '@/stores/configStore'
import { IconDrive } from '@/common/icons'
import { invoke } from '@tauri-apps/api/core'
import TreeFolderNode from './TreeFolderNode.vue'

const navigationStore = useNavigationStore()
const galleryStore = useGalleryStore()
const configStore = useConfigStore()
const expandedNodes = reactive({})

const dragOverDrivePath = ref('')

const visibleDrives = computed(() => {
  const hidden = configStore.settings.hiddenDrives || []
  return navigationStore.drives.filter(d => !hidden.includes(d.path))
})

async function handleDriveDrop(e, destPath) {
  dragOverDrivePath.value = ''
  try {
    const data = e.dataTransfer.getData('text/plain')
    if (!data) return
    const paths = JSON.parse(data)
    if (!Array.isArray(paths) || paths.length === 0) return

    for (const src of paths) {
      if (src === destPath) continue
      const lastSlash = Math.max(src.lastIndexOf('\\'), src.lastIndexOf('/'))
      const fileName = lastSlash !== -1 ? src.substring(lastSlash + 1) : src
      const dest = `${destPath}${destPath.endsWith('\\') || destPath.endsWith('/') ? '' : '\\'}${fileName}`
      if (src.toLowerCase() === dest.toLowerCase()) continue

      await invoke('cross_move', { src, dest })
    }

    await navigationStore.navigateTo(navigationStore.currentPath)
    galleryStore.setFiles(navigationStore.folders)
  } catch (err) {
    console.error('Drive drop failed:', err)
  }
}

async function navigateTo(path) {
  if (path) {
    const isDrive = navigationStore.drives.some(d => d.path === path)
    if (isDrive) {
      expandedNodes[path] = !expandedNodes[path]
      if (expandedNodes[path]) {
        await navigationStore.expandTreeFolder(path)
      }
    } else {
      if (!expandedNodes[path]) {
        expandedNodes[path] = true
        await navigationStore.expandTreeFolder(path)
      }
    }
    await navigationStore.navigateTo(path)
    galleryStore.setFiles(navigationStore.folders)
  }
}

async function toggleDrive(path) {
  expandedNodes[path] = !expandedNodes[path]
  if (expandedNodes[path]) {
    await navigationStore.expandTreeFolder(path)
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
.chevron {
  font-family: monospace;
}
</style>