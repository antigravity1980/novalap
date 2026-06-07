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

    <ContextMenu
      ref="contextMenuRef"
      :menuItems="recolorMenuItems"
      :smallIcon="true"
      style="display: none;"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useNavigationStore } from '../store'
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
const galleryStore = useGalleryStore()
const configStore = useConfigStore()
const isExpanded = ref(false)
const children = ref(props.folder.children || [])
const hasChildren = computed(() => props.folder.has_subfolders)
const isActive = computed(() => props.folder.path === navigationStore.currentPath)
const contextMenuRef = ref(null)
const dragOverFolderPath = ref('')

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
      label: isFav ? 'Удалить из избранного' : 'Добавить в избранное',
      action: () => configStore.toggleFavorite(props.folder.path)
    },
    {
      label: 'Перекрасить папку',
      children: [
        { label: '⭐ Важная (Звезда)', action: () => setFolderIcon('I1.ico') },
        { label: 'По умолчанию', action: () => setFolderIcon(null) },
      { label: 'Папка 01', action: () => setFolderIcon('01.ico') },
      { label: 'Папка 02', action: () => setFolderIcon('02.ico') },
      { label: 'Папка 03', action: () => setFolderIcon('03.ico') },
      { label: 'Папка 04', action: () => setFolderIcon('04.ico') },
      { label: 'Папка 05', action: () => setFolderIcon('05.ico') },
      { label: 'Папка 06', action: () => setFolderIcon('06.ico') },
      { label: 'Папка 07', action: () => setFolderIcon('07.ico') },
      { label: 'Папка 08', action: () => setFolderIcon('08.ico') },
      { label: 'Папка 09', action: () => setFolderIcon('09.ico') },
      { label: 'Папка 10', action: () => setFolderIcon('10.ico') },
      { label: 'Папка 11', action: () => setFolderIcon('11.ico') },
      { label: 'Папка 12', action: () => setFolderIcon('12.ico') },
      { label: 'Папка 15', action: () => setFolderIcon('15.ico') },
    ]
  }
]
})

function handleContextMenu(e) {
  contextMenuRef.value?.open(e.clientX, e.clientY)
}

function setFolderIcon(iconName) {
  configStore.setFolderIcon(props.folder.path, iconName)
}

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