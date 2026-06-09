<template>
  <div class="tree-folder-node select-none">
    <div
      class="tree-item flex items-center gap-1.5 px-2.5 py-1.5 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150"
      :class="{
        'bg-primary/10 text-primary font-bold shadow-sm': isActive && dragOverFolderPath !== folder.path,
        'text-base-content/70 hover:bg-base-100/30 hover:text-base-content': !isActive && dragOverFolderPath !== folder.path,
        'bg-secondary/20 text-secondary border border-dashed border-secondary/50': dragOverFolderPath === folder.path,
      }"
      @click="navigate"
      @contextmenu.prevent.stop="handleContextMenu"
      @dragover.prevent="dragOverFolderPath = folder.path"
      @dragenter.prevent="dragOverFolderPath = folder.path"
      @dragleave="dragOverFolderPath = ''"
      @drop.prevent="handleFolderDrop($event, folder.path)"
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
      <span class="text-sm shrink-0 flex items-center justify-center w-4 h-4">
        <img :src="folderIconUrl" class="w-4 h-4 object-contain select-none pointer-events-none" />
      </span>

      <span class="truncate flex-1">{{ folder.name }}</span>
    </div>

    <!-- Children list — читаем из стора, computed реактивно -->
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

    <!-- New folder input -->
    <div v-if="showNewFolderInput" class="ml-7 mt-1 flex items-center gap-1">
      <input
        ref="newFolderInputRef"
        v-model="newFolderName"
        type="text"
        class="flex-1 px-2 py-1 text-xs bg-base-100 border border-primary/30 rounded outline-none focus:border-primary"
        placeholder="Имя папки"
        @keydown.enter.prevent="confirmNewFolder"
        @keydown.escape="cancelNewFolder"
      />
      <button
        class="px-2 py-1 text-xs bg-primary/20 hover:bg-primary/30 rounded border border-primary/30"
        @click="confirmNewFolder"
        @mousedown.prevent
      >OK</button>
    </div>

    <ContextMenu
      ref="contextMenuRef"
      :menuItems="recolorMenuItems"
      :smallIcon="true"
      style="display: none;"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue'
import { useNavigationStore } from '../store'
import { useClipboardStore } from '../stores/clipboardStore'
import { useGalleryStore } from '../../gallery/store'
import { useConfigStore } from '@/stores/configStore'
import { getAssetSrc } from '@/common/utils'
import { invoke } from '@tauri-apps/api/core'
import ContextMenu from '@/components/ContextMenu.vue'

const props = defineProps({
  folder: { type: Object, required: true },
  activePath: { type: String, default: '' },
})

const emit = defineEmits(['navigate', 'expand'])

const navigationStore = useNavigationStore()
const clipboardStore = useClipboardStore()
const galleryStore = useGalleryStore()
const configStore = useConfigStore()
const isExpanded = ref(false)
const children = computed(() => navigationStore.treeFolders[props.folder.path] || [])
const hasChildren = computed(() => props.folder.has_subfolders)
const isActive = computed(() => props.folder.path === navigationStore.currentPath)
const contextMenuRef = ref(null)
const dragOverFolderPath = ref('')
const showNewFolderInput = ref(false)
const newFolderName = ref('')
const newFolderInputRef = ref(null)

async function handleFolderDrop(e, destPath) {
  dragOverFolderPath.value = ''
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
    console.error('Folder drop failed:', err)
  }
}

const folderIconUrl = computed(() => {
  const customIcon = configStore.folderIcons?.[props.folder.path]
  if (customIcon) {
    return getAssetSrc(`D:\\NovaLAP\\Folder\\${customIcon}`)
  }
  const defaultIcon = '14.ico'
  return getAssetSrc(`D:\\NovaLAP\\Folder\\${defaultIcon}`)
})

const recolorMenuItems = computed(() => {
  const isFav = configStore.settings.favorites?.includes(props.folder.path)
  return [
    {
      label: 'Создать папку',
      action: () => startNewFolder()
    },
    {
      label: '-',
    },
    {
      label: isFav ? 'Удалить из избранного' : 'Добавить в избранное',
      action: () => configStore.toggleFavorite(props.folder.path)
    },
    {
      label: 'Перекрасить папку',
      grid: true,
      children: [
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\14.ico'), tooltip: 'По умолчанию', action: () => setFolderIcon(null) },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\I1.ico'), tooltip: 'Важная (Звезда)', action: () => setFolderIcon('I1.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\01.ico'), tooltip: 'Папка 01', action: () => setFolderIcon('01.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\02.ico'), tooltip: 'Папка 02', action: () => setFolderIcon('02.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\03.ico'), tooltip: 'Папка 03', action: () => setFolderIcon('03.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\04.ico'), tooltip: 'Папка 04', action: () => setFolderIcon('04.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\05.ico'), tooltip: 'Папка 05', action: () => setFolderIcon('05.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\06.ico'), tooltip: 'Папка 06', action: () => setFolderIcon('06.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\07.ico'), tooltip: 'Папка 07', action: () => setFolderIcon('07.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\08.ico'), tooltip: 'Папка 08', action: () => setFolderIcon('08.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\09.ico'), tooltip: 'Папка 09', action: () => setFolderIcon('09.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\10.ico'), tooltip: 'Папка 10', action: () => setFolderIcon('10.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\11.ico'), tooltip: 'Папка 11', action: () => setFolderIcon('11.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\12.ico'), tooltip: 'Папка 12', action: () => setFolderIcon('12.ico') },
        { iconUrl: getAssetSrc('D:\\NovaLAP\\Folder\\15.ico'), tooltip: 'Папка 15', action: () => setFolderIcon('15.ico') },
      ]
    },
    {
      label: '-',
    },
    {
      label: 'Копировать',
      action: () => clipboardStore.copy(props.folder.path)
    },
    {
      label: 'Вырезать',
      action: () => clipboardStore.cut(props.folder.path)
    },
    {
      label: 'Вставить',
      disabled: !clipboardStore.hasItems,
      action: () => handlePaste()
    },
    {
      label: '-',
    },
    {
      label: 'Удалить',
      action: () => handleDelete()
    },
  ]
})

function handleContextMenu(e) {
  contextMenuRef.value?.open(e.clientX, e.clientY)
}

function setFolderIcon(iconName) {
  configStore.setFolderIcon(props.folder.path, iconName)
}

function startNewFolder() {
  showNewFolderInput.value = true
  newFolderName.value = ''
  nextTick(() => {
    newFolderInputRef.value?.focus()
  })
}

async function confirmNewFolder() {
  const name = newFolderName.value.trim()
  showNewFolderInput.value = false
  if (!name) return

  try {
    const folderPath = props.folder.path.endsWith('\\') || props.folder.path.endsWith('/')
      ? props.folder.path + name
      : props.folder.path + '\\' + name
    await invoke('mkdir_folder', { path: folderPath })
    // Перезагружаем родительскую папку (где создали подпапку)
    await navigationStore.refreshTreeFolder(props.folder.path)
    // Автоматически разворачиваем родителя
    if (!isExpanded.value) {
      isExpanded.value = true
    }
  } catch (err) {
    console.error('Failed to create folder:', err)
  }
}

function cancelNewFolder() {
  showNewFolderInput.value = false
  newFolderName.value = ''
}

async function handlePaste() {
  try {
    await clipboardStore.paste(props.folder.path)
    // Перезагружаем целевую папку (и родителя если был cut)
    await navigationStore.refreshTreeFolder(props.folder.path)
    // Если cut — перезагружаем родителя источника
    if (clipboardStore.mode === 'cut' && clipboardStore.items.length) {
      for (const src of clipboardStore.items) {
        const parentSrc = src.substring(0, src.lastIndexOf('\\'))
        if (parentSrc && parentSrc !== props.folder.path) {
          await navigationStore.refreshTreeFolder(parentSrc)
        }
      }
    }
    isExpanded.value = true
    // Также обновить галерею если мы в этой папке
    if (navigationStore.currentPath === props.folder.path) {
      await navigationStore.navigateTo(navigationStore.currentPath)
      galleryStore.setFiles(navigationStore.folders)
    }
  } catch (err) {
    console.error('Paste failed:', err)
  }
}

async function handleDelete() {
  try {
    const pathToDelete = props.folder.path
    const parentPath = pathToDelete.substring(0, pathToDelete.lastIndexOf('\\'))
    await invoke('delete_file_system', { path: pathToDelete })
    // Перезагружаем родителя
    if (parentPath) {
      await navigationStore.refreshTreeFolder(parentPath)
    }
    // Если мы в удалённой папке — перейти в родителя
    if (navigationStore.currentPath === pathToDelete || navigationStore.currentPath.startsWith(pathToDelete + '\\')) {
      await navigationStore.navigateTo(parentPath || 'C:\\')
      galleryStore.setFiles(navigationStore.folders)
    }
  } catch (err) {
    console.error('Delete failed:', err)
  }
}

async function toggle() {
  if (!hasChildren.value) return
  isExpanded.value = !isExpanded.value
  if (isExpanded.value && children.value.length === 0) {
    emit('expand', props.folder.path)
    await navigationStore.expandTreeFolder(props.folder.path)
  }
}

async function navigate() {
  emit('navigate', props.folder.path)
  await toggle()
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