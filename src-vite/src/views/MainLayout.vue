<template>
  <div class="main-layout flex h-screen w-screen overflow-hidden bg-base-100">
    <!-- Left sidebar: Explorer Tree -->
    <aside class="sidebar w-64 flex-shrink-0 border-r border-base-200 bg-base-200/30">
      <ExplorerTree />
    </aside>

    <!-- Main content area -->
    <main class="main-content flex-1 flex flex-col overflow-hidden">
      <!-- Top bar: Breadcrumb + Actions -->
      <header class="top-bar flex items-center justify-between px-4 py-2 border-b border-base-200 bg-base-100/80 backdrop-blur">
        <!-- Breadcrumb navigation -->
        <div class="breadcrumb flex items-center gap-1 text-sm">
          <button class="btn btn-ghost btn-xs" @click="goBack" :disabled="!navigationStore.canGoBack">←</button>
          <button class="btn btn-ghost btn-xs" @click="goForward" :disabled="!navigationStore.canGoForward">→</button>

          <span class="mx-1 text-base-content/30">|</span>

          <template v-for="(part, i) in breadcrumbs" :key="i">
            <button
              class="breadcrumb-item hover:text-primary transition-colors"
              :class="{ 'text-primary font-medium': i === breadcrumbs.length - 1 }"
              @click="navigateTo(part.path)"
            >
              {{ part.name }}
            </button>
            <span v-if="i < breadcrumbs.length - 1" class="text-base-content/30 mx-0.5">/</span>
          </template>
        </div>

        <!-- Quick actions -->
        <div class="actions flex items-center gap-2">
          <button class="btn btn-ghost btn-sm btn-square" title="New Folder" @click="createFolder">
            📁+
          </button>
          <button class="btn btn-ghost btn-sm btn-square" title="Refresh" @click="navigationStore.refresh()">
            🔄
          </button>
          <button class="btn btn-ghost btn-sm btn-square" title="Select All" @click="galleryStore.selectAll()">
            ☑
          </button>
          <button class="btn btn-ghost btn-sm btn-square" title="Undo (Ctrl+Z)" @click="galleryStore.undo()" :disabled="!galleryStore.canUndo">
            ↩
          </button>
          <button class="btn btn-ghost btn-sm btn-square" title="Redo (Ctrl+Shift+Z)" @click="galleryStore.redo()" :disabled="!galleryStore.canRedo">
            ↪
          </button>
        </div>
      </header>

      <!-- Filter bar -->
      <div class="filter-bar-container px-4 py-1">
        <FilterBar />
      </div>

      <!-- Gallery grid -->
      <div class="gallery-area flex-1 overflow-hidden">
        <GalleryGrid @open-quick-look="openQuickLook" />
      </div>
    </main>

    <!-- Quick Look overlay -->
    <QuickLook
      :visible="quickLookVisible"
      :files="filteredFiles"
      :initial-index="quickLookIndex"
      @update:visible="quickLookVisible = $event"
    />

    <!-- Status bar -->
    <footer class="status-bar fixed bottom-0 left-0 right-0 h-6 bg-base-200 text-xs text-base-content/50 flex items-center px-3 gap-4">
      <span>{{ navigationStore.folders.length }} items</span>
      <span v-if="navigationStore.currentPath">{{ navigationStore.currentPath }}</span>
      <span v-if="galleryStore.selectedIds.length > 0" class="text-primary">
        {{ galleryStore.selectedIds.length }} selected
      </span>
      <span class="ml-auto">
        Zoom: {{ Math.round(galleryStore.zoomLevel * 100) }}%
      </span>
    </footer>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNavigationStore } from '@/modules/navigation/store'
import { useGalleryStore } from '@/modules/gallery/store'
import ExplorerTree from '@/modules/navigation/components/ExplorerTree.vue'
import FilterBar from '@/modules/gallery/components/FilterBar.vue'
import GalleryGrid from '@/modules/gallery/components/GalleryGrid.vue'
import QuickLook from '@/modules/viewer/components/QuickLook.vue'

const navigationStore = useNavigationStore()
const galleryStore = useGalleryStore()

// Quick Look state
const quickLookVisible = ref(false)
const quickLookIndex = ref(0)

// Breadcrumbs
const breadcrumbs = computed(() => {
  if (!navigationStore.currentPath) return []
  return navigationStore.currentPath
    .split('\\')
    .filter(Boolean)
    .reduce((acc, part, i, arr) => {
      const path = arr.slice(0, i + 1).join('\\')
      acc.push({ name: part, path })
      return acc
    }, [])
})

// Filtered files for Quick Look navigation
const filteredFiles = computed(() => galleryStore.displayedFiles)

// Navigation
async function navigateTo(path) {
  await navigationStore.navigateTo(path)
  galleryStore.setFiles(navigationStore.folders)
}

async function goBack() {
  await navigationStore.goBack()
  galleryStore.setFiles(navigationStore.folders)
}

async function goForward() {
  await navigationStore.goForward()
  galleryStore.setFiles(navigationStore.folders)
}

// Quick Look
function openQuickLook(file) {
  const index = filteredFiles.value.findIndex(f => f.path === file.path)
  if (index >= 0) {
    quickLookIndex.value = index
    quickLookVisible.value = true
  }
}

// Create folder
async function createFolder() {
  const folderName = prompt('Enter folder name:')
  if (folderName && navigationStore.currentPath) {
    const newPath = `${navigationStore.currentPath}\\${folderName}`
    try {
      await invoke('create_folder', { path: newPath })
      await navigationStore.refresh()
    } catch (error) {
      console.error('Failed to create folder:', error)
    }
  }
}

// Keyboard shortcuts
function handleKeyDown(event) {
  // Ctrl+Z = Undo
  if (event.ctrlKey && !event.shiftKey && event.key === 'z') {
    event.preventDefault()
    galleryStore.undo()
  }
  // Ctrl+Shift+Z = Redo
  if (event.ctrlKey && event.shiftKey && event.key === 'z') {
    event.preventDefault()
    galleryStore.redo()
  }
  // Ctrl+A = Select all
  if (event.ctrlKey && event.key === 'a') {
    event.preventDefault()
    galleryStore.selectAll()
  }
  // Space = Quick Look
  if (event.key === ' ' && !quickLookVisible.value) {
    const target = event.target
    if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
      event.preventDefault()
      // Open Quick Look for first selected or first file
      if (galleryStore.selectedIds.length > 0) {
        const firstSelected = galleryStore.selectedIds[0]
        const file = filteredFiles.value.find(f => f.path === firstSelected)
        if (file) openQuickLook(file)
      } else if (filteredFiles.value.length > 0) {
        openQuickLook(filteredFiles.value[0])
      }
    }
  }
}

// Init
onMounted(async () => {
  document.addEventListener('keydown', handleKeyDown)
  await navigationStore.loadDrives()

  // Start in a default location (user's home/pictures)
  const defaultPath = await invoke('get_file_info', { path: '' }).catch(() => null)
  const homeDrive = navigationStore.drives.find(d => d.path.startsWith('C:'))
  if (homeDrive) {
    await navigateTo(homeDrive.path)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
.main-layout {
  font-size: 14px;
}
.sidebar {
  overflow-y: auto;
  z-index: 10;
}
.main-content {
  padding-bottom: 24px; /* space for status bar */
}
.top-bar {
  min-height: 40px;
}
.breadcrumb-item {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.status-bar {
  z-index: 20;
}
</style>