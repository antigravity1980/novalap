<template>
  <div class="explorer-tree py-2 h-full" @contextmenu.prevent.stop="handleEmptySpaceContextMenu($event)">
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
          @contextmenu.prevent.stop="handleDriveContextMenu($event, drive.path)"
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

        <!-- New folder input for drive -->
        <div v-if="showNewFolderOnDrive && currentContextDrivePath === drive.path" class="ml-7 mt-1 flex items-center gap-1">
          <input
            ref="newFolderInputRef"
            v-model="newFolderName"
            type="text"
            class="flex-1 px-2 py-1 text-xs bg-base-100 border border-primary/30 rounded outline-none focus:border-primary"
            placeholder="Имя папки"
            @keydown.enter.prevent="confirmNewFolderOnDrive"
            @keydown.escape="cancelNewFolderOnDrive"
          />
          <button
            class="px-2 py-1 text-xs bg-primary/20 hover:bg-primary/30 rounded border border-primary/30"
            @click="confirmNewFolderOnDrive"
            @mousedown.prevent
          >OK</button>
        </div>
      </div>
    </div>

    <ContextMenu
      ref="contextMenuRef"
      :menuItems="activeMenuItems"
      :smallIcon="true"
      style="display: none;"
    />
  </div>
</template>

<script setup>
import { reactive, onMounted, watch, computed, ref, nextTick } from 'vue'
import { useNavigationStore } from '../store'
import { useClipboardStore } from '../stores/clipboardStore'
import { useGalleryStore } from '../../gallery/store'
import { useConfigStore } from '@/stores/configStore'
import { IconDrive } from '@/common/icons'
import { invoke } from '@tauri-apps/api/core'
import TreeFolderNode from './TreeFolderNode.vue'
import ContextMenu from '@/components/ContextMenu.vue'

function buildNewFolderEntry(path) {
  path = navigationStore.normalizePath(path)
  const normalizedPath = path.replace(/[\\/]+$/, '')
  const lastSlash = Math.max(normalizedPath.lastIndexOf('\\'), normalizedPath.lastIndexOf('/'))
  const name = lastSlash >= 0 ? normalizedPath.substring(lastSlash + 1) : normalizedPath
  return {
    name,
    path,
    is_dir: true,
    has_subfolders: false,
  }
}

const navigationStore = useNavigationStore()
const clipboardStore = useClipboardStore()
const galleryStore = useGalleryStore()
const configStore = useConfigStore()
const expandedNodes = reactive({})

const dragOverDrivePath = ref('')
const contextMenuRef = ref(null)
const currentContextDrivePath = ref('')
const showNewFolderOnDrive = ref(false)
const newFolderName = ref('')
const newFolderInputRef = ref(null)

const contextMenuType = ref('drive')

const visibleDrives = computed(() => {
  const hidden = configStore.settings.hiddenDrives || []
  return navigationStore.drives.filter(d => !hidden.includes(d.path))
})

const activeMenuItems = computed(() => {
  return contextMenuType.value === 'drive' ? driveMenuItems.value : emptySpaceMenuItems.value
})

const driveMenuItems = computed(() => {
  return [
    {
      label: 'Создать папку',
      action: () => startNewFolderOnDrive()
    },
    {
      label: 'Вставить',
      disabled: !clipboardStore.hasItems,
      action: () => handlePasteOnDrive()
    },
  ]
})

const emptySpaceMenuItems = computed(() => {
  return [
    {
      label: 'Создать папку',
      disabled: !navigationStore.currentPath,
      action: () => createFolderInCurrentDir()
    },
    {
      label: 'Вставить',
      disabled: !navigationStore.currentPath || !clipboardStore.hasItems,
      action: () => handlePasteInCurrentDir()
    },
  ]
})

function handleDriveContextMenu(e, drivePath) {
  drivePath = navigationStore.normalizePath(drivePath)
  currentContextDrivePath.value = drivePath
  contextMenuType.value = 'drive'
  contextMenuRef.value?.open(e.clientX, e.clientY)
}

function handleEmptySpaceContextMenu(e) {
  if (e.target.closest('.tree-item') || e.target.closest('.chevron')) return
  contextMenuType.value = 'empty'
  contextMenuRef.value?.open(e.clientX, e.clientY)
}

async function createFolderInCurrentDir() {
  if (!navigationStore.currentPath) return
  const separator = navigationStore.currentPath.includes('/') ? '/' : '\\'

  let name = 'Новая папка'
  let counter = 1
  const checkNameExists = (n) => {
    return navigationStore.folders.some(f => f.name.toLowerCase() === n.toLowerCase())
  }

  while (checkNameExists(name)) {
    counter++
    name = `Новая папка (${counter})`
  }

  const newPath = navigationStore.currentPath.endsWith(separator)
    ? navigationStore.currentPath + name
    : navigationStore.currentPath + separator + name

  try {
    await invoke('mkdir_folder', { path: newPath })

    const newFolder = {
      name,
      path: newPath,
      is_dir: true,
      is_file: false,
      size: 0,
      modified: new Date().toISOString(),
      created: new Date().toISOString(),
      extension: null,
      resolution: null,
      dir_count: 0,
      file_count: 0,
      ai_source: null,
    }

    navigationStore.folders = [newFolder, ...navigationStore.folders.filter(f => f.path !== newPath)]
    galleryStore.upsertFile(newFolder, { pinToTop: true })

    const treeEntry = buildNewFolderEntry(newPath)
    const currentChildren = navigationStore.treeFolders[navigationStore.currentPath] || []
    navigationStore.treeFolders[navigationStore.currentPath] = [treeEntry, ...currentChildren.filter(f => f.path !== newPath)]
    expandedNodes[navigationStore.currentPath] = true
    navigationStore.pendingTreeRenamePath = newPath
    galleryStore.renamingPath = newPath
  } catch (err) {
    console.error('Failed to create folder from sidebar tree:', err)
  }
}

async function handlePasteInCurrentDir() {
  const destPath = navigationStore.currentPath
  if (!destPath) return
  try {
    await clipboardStore.paste(destPath)
    await navigationStore.refreshTreeFolder(destPath)
    expandedNodes[destPath] = true

    if (clipboardStore.mode === 'cut' && clipboardStore.items.length) {
      for (const src of clipboardStore.items) {
        const parentSrc = src.substring(0, src.lastIndexOf('\\'))
        if (parentSrc && parentSrc !== destPath) {
          await navigationStore.refreshTreeFolder(parentSrc)
        }
      }
    }

    await navigationStore.navigateTo(destPath)
    galleryStore.setFiles(navigationStore.folders)
  } catch (err) {
    console.error('Paste in current dir failed:', err)
  }
}

function startNewFolderOnDrive() {
  showNewFolderOnDrive.value = true
  newFolderName.value = ''
  nextTick(() => {
    newFolderInputRef.value?.focus()
  })
}

async function confirmNewFolderOnDrive() {
  const name = newFolderName.value.trim()
  showNewFolderOnDrive.value = false
  if (!name) return

  try {
    const folderPath = currentContextDrivePath.value.endsWith('\\')
      ? currentContextDrivePath.value + name
      : currentContextDrivePath.value + '\\' + name
    await invoke('mkdir_folder', { path: folderPath })

    const treeEntry = buildNewFolderEntry(folderPath)
    const currentChildren = navigationStore.treeFolders[currentContextDrivePath.value] || []
    navigationStore.treeFolders[currentContextDrivePath.value] = [treeEntry, ...currentChildren.filter(f => f.path !== folderPath)]
    expandedNodes[currentContextDrivePath.value] = true
    navigationStore.pendingTreeRenamePath = folderPath

    if (navigationStore.currentPath === currentContextDrivePath.value) {
      const newFolder = {
        name,
        path: folderPath,
        is_dir: true,
        is_file: false,
        size: 0,
        modified: new Date().toISOString(),
        created: new Date().toISOString(),
        extension: null,
        resolution: null,
        dir_count: 0,
        file_count: 0,
        ai_source: null,
      }
      navigationStore.folders = [newFolder, ...navigationStore.folders.filter(f => f.path !== folderPath)]
      galleryStore.upsertFile(newFolder, { pinToTop: true })
      galleryStore.renamingPath = folderPath
    }
  } catch (err) {
    console.error('Failed to create folder on drive:', err)
  }
}

function cancelNewFolderOnDrive() {
  showNewFolderOnDrive.value = false
  newFolderName.value = ''
}

async function handlePasteOnDrive() {
  try {
    await clipboardStore.paste(currentContextDrivePath.value)
    // Перезагружаем диск
    await navigationStore.refreshTreeFolder(currentContextDrivePath.value)
    expandedNodes[currentContextDrivePath.value] = true
    // Перезагружаем родителей источников при cut
    if (clipboardStore.mode === 'cut' && clipboardStore.items.length) {
      for (const src of clipboardStore.items) {
        const parentSrc = src.substring(0, src.lastIndexOf('\\'))
        if (parentSrc && parentSrc !== currentContextDrivePath.value) {
          await navigationStore.refreshTreeFolder(parentSrc)
        }
      }
    }
    // Обновить галерею если мы в этом диске
    if (navigationStore.currentPath === currentContextDrivePath.value) {
      await navigationStore.navigateTo(navigationStore.currentPath)
      galleryStore.setFiles(navigationStore.folders)
    }
  } catch (err) {
    console.error('Paste on drive failed:', err)
  }
}

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
  if (navigationStore.currentPath) {
    const drive = navigationStore.drives.find(d => navigationStore.currentPath.startsWith(d.path))
    if (drive) {
      expandedNodes[drive.path] = true
    }
  }
})

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
