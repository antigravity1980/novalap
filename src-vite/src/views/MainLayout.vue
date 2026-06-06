<template>
  <div class="main-layout flex h-screen w-screen overflow-hidden bg-base-300 text-base-content select-none font-sans">
    <!-- Activity Bar (Sleek Left Sidebar tabs) -->
    <aside class="w-16 flex-shrink-0 bg-base-200 border-r border-base-100/30 flex flex-col items-center py-4 justify-between z-20">
      <div class="flex flex-col gap-4 w-full items-center">
        <!-- Explorer tab icon -->
        <button
          class="w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 hover:bg-base-100/50"
          :class="activeTab === 'explorer' ? 'bg-primary text-primary-content shadow-lg shadow-primary/20' : 'text-base-content/60'"
          title="Explorer"
          @click="activeTab = 'explorer'"
        >
          <IconFolders class="w-5 h-5" />
        </button>

        <!-- Trash tab icon -->
        <button
          class="w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 hover:bg-base-100/50 relative"
          :class="activeTab === 'trash' ? 'bg-primary text-primary-content shadow-lg shadow-primary/20' : 'text-base-content/60'"
          title="Trash Bin"
          @click="activeTab = 'trash'"
        >
          <IconTrash class="w-5 h-5" />
          <span v-if="trashCount > 0" class="absolute top-1 right-1 w-2.5 h-2.5 bg-error rounded-full ring-2 ring-base-200"></span>
        </button>
      </div>

      <!-- Settings icon -->
      <button
        class="w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 hover:bg-base-100/50 text-base-content/60 hover:text-base-content"
        title="Settings"
        @click="openSettings"
      >
        <IconSettings class="w-5 h-5 animate-hover-spin" />
      </button>
    </aside>

    <!-- Navigation & Explorer sidebar panel (Collapsible) -->
    <aside
      v-if="showSidebar"
      class="sidebar-panel border-r border-base-100/30 bg-base-200/50 backdrop-blur flex flex-col overflow-hidden transition-all duration-300"
      :style="{ width: configStore.leftPanel.width + 'px' }"
    >
      <!-- Explorer panel -->
      <div v-if="activeTab === 'explorer'" class="flex flex-col h-full overflow-hidden">
        <div class="p-4 border-b border-base-100/20 bg-base-200/20 flex items-center justify-between shrink-0">
          <span class="text-xs uppercase font-bold tracking-wider text-base-content/40">Explorer</span>
          <!-- Drives selector -->
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-xs text-xs font-semibold gap-1 px-2 py-0.5 rounded bg-base-300/40 border-base-200/30 hover:bg-base-100/30">
              Drives ▾
            </label>
            <ul tabindex="0" class="dropdown-content menu p-2 shadow-2xl bg-base-300 border border-base-200/30 rounded-box w-52 z-30 text-xs mt-1">
              <li v-for="drive in navigationStore.drives" :key="drive.path">
                <a @click="navigateTo(drive.path)">
                  <span class="text-sm">💾</span>
                  <span class="font-medium">{{ drive.name }}</span>
                  <span class="opacity-50 font-mono text-[10px]">({{ drive.path }})</span>
                </a>
              </li>
            </ul>
          </div>
        </div>
        <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
          <ExplorerTree />
        </div>
      </div>

      <!-- Trash Bin panel -->
      <div v-else-if="activeTab === 'trash'" class="flex flex-col h-full overflow-hidden">
        <div class="p-4 border-b border-base-100/20 bg-base-200/20 flex items-center justify-between shrink-0">
          <span class="text-xs uppercase font-bold tracking-wider text-base-content/40">Trash Bin</span>
          <button
            class="btn btn-ghost btn-xs text-error font-semibold hover:bg-error/10"
            :disabled="trashItems.length === 0"
            @click="clearTrash"
          >
            Empty Trash
          </button>
        </div>

        <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
          <div v-if="trashItems.length === 0" class="flex flex-col items-center justify-center py-12 text-center text-base-content/30 space-y-2">
            <span class="text-3xl">🗑️</span>
            <span class="text-xs">Trash is empty</span>
          </div>
          <div
            v-for="item in trashItems"
            :key="item.trashPath"
            class="p-2.5 rounded-lg bg-base-300/35 border border-base-200/20 flex flex-col justify-between hover:bg-base-100/20 transition-all duration-150 relative group"
          >
            <div class="pr-6">
              <p class="text-xs font-semibold text-base-content/80 truncate" :title="getFileName(item.originalPath)">
                {{ getFileName(item.originalPath) }}
              </p>
              <p class="text-[10px] text-base-content/40 truncate mt-0.5" :title="item.originalPath">
                {{ item.originalPath }}
              </p>
            </div>
            <div class="flex items-center justify-between mt-2 text-[10px] text-base-content/50">
              <span>{{ formatBytes(item.size) }}</span>
              <div class="flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                <button class="btn btn-primary btn-xs px-2 h-5 min-h-[20px] rounded text-[10px]" @click="restoreTrashFile(item.trashPath)">
                  Restore
                </button>
                <button class="btn btn-ghost btn-xs px-2 h-5 min-h-[20px] rounded text-[10px] text-error hover:bg-error/10" @click="deleteTrashFilePermanently(item)">
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- Drag splitter for Left Sidebar -->
    <div
      v-if="showSidebar"
      class="w-1 shrink-0 cursor-col-resize hover:bg-primary/50 transition-colors z-20"
      :class="{ 'bg-primary/40': isDraggingLeftSplitter }"
      @mousedown="startDraggingLeftSplitter"
    ></div>

    <!-- Main Workspace -->
    <main class="flex-1 flex flex-col overflow-hidden bg-base-300">
      <!-- Breadcrumbs and Navigation header -->
      <header class="h-14 flex items-center justify-between px-6 border-b border-base-100/30 bg-base-200/35 backdrop-blur shrink-0 z-10">
        <!-- Path and Navigation Arrows -->
        <div class="flex items-center gap-4 min-w-0">
          <div class="flex gap-1 shrink-0">
            <button
              class="btn btn-ghost btn-xs btn-circle text-base-content/60 hover:text-base-content hover:bg-base-100/40"
              @click="goBack"
              :disabled="!navigationStore.canGoBack"
            >
              ‹
            </button>
            <button
              class="btn btn-ghost btn-xs btn-circle text-base-content/60 hover:text-base-content hover:bg-base-100/40"
              @click="goForward"
              :disabled="!navigationStore.canGoForward"
            >
              ›
            </button>
          </div>

          <div class="divider divider-horizontal mx-0 shrink-0"></div>

          <!-- Breadcrumbs -->
          <div class="breadcrumbs text-xs font-semibold text-base-content/80 overflow-x-auto no-scrollbar whitespace-nowrap py-1">
            <ul>
              <li v-for="(crumb, i) in breadcrumbs" :key="crumb.path">
                <a @click="navigateTo(crumb.path)" class="hover:text-primary transition-colors cursor-pointer text-base-content/70">
                  {{ crumb.name }}
                </a>
              </li>
            </ul>
          </div>
        </div>

        <!-- Quick actions toolbar -->
        <div class="flex items-center gap-2 shrink-0">
          <!-- Folder Action -->
          <button
            v-if="activeTab === 'explorer' && navigationStore.currentPath"
            class="btn btn-ghost btn-sm btn-square rounded-lg text-base-content/60 hover:text-base-content hover:bg-base-100/40"
            title="New Folder"
            @click="createFolder"
          >
            📁+
          </button>

          <!-- Compare Action -->
          <button
            class="btn btn-ghost btn-sm rounded-lg text-xs font-bold gap-1 text-base-content/60 hover:text-base-content hover:bg-base-100/40"
            :disabled="galleryStore.selectedIds.length < 2"
            title="Compare selected images (C)"
            @click="openCompare"
          >
            ⚖️ Compare
          </button>

          <!-- Batch Operations -->
          <button
            class="btn btn-ghost btn-sm rounded-lg text-xs font-bold gap-1 text-base-content/60 hover:text-base-content hover:bg-base-100/40"
            :disabled="galleryStore.selectedIds.length === 0"
            title="Batch processing selected items"
            @click="batchOperationsVisible = true"
          >
            🛠️ Batch
          </button>

          <!-- Refresh -->
          <button
            class="btn btn-ghost btn-sm btn-square rounded-lg text-base-content/60 hover:text-base-content hover:bg-base-100/40"
            title="Refresh"
            @click="refreshData"
          >
            🔄
          </button>

          <div class="divider divider-horizontal mx-1"></div>

          <!-- Info / Inspector toggle -->
          <button
            class="btn btn-sm rounded-lg text-xs font-bold gap-1"
            :class="configStore.rightPanel.show ? 'btn-primary shadow-lg shadow-primary/10' : 'btn-ghost text-base-content/60 hover:text-base-content hover:bg-base-100/40'"
            @click="toggleInspector"
          >
            ℹ️ Info
          </button>
        </div>
      </header>

      <!-- Filter bar container -->
      <div class="px-6 py-2 border-b border-base-100/20 bg-base-300/40 shrink-0">
        <FilterBar />
      </div>

      <!-- Main gallery area -->
      <div class="flex-1 overflow-hidden relative">
        <GalleryGrid @open-quick-look="openQuickLook" />
      </div>
    </main>

    <!-- Drag splitter for Right Inspector -->
    <div
      v-if="configStore.rightPanel.show"
      class="w-1 shrink-0 cursor-col-resize hover:bg-primary/50 transition-colors z-20"
      :class="{ 'bg-primary/40': isDraggingRightSplitter }"
      @mousedown="startDraggingRightSplitter"
    ></div>

    <!-- Right Collapsible Inspector Panel -->
    <aside
      v-if="configStore.rightPanel.show"
      class="inspector-panel border-l border-base-100/30 bg-base-200/50 backdrop-blur flex flex-col overflow-hidden shrink-0 transition-all duration-300 z-10"
      :style="{ width: configStore.rightPanel.width + 'px' }"
    >
      <!-- Single file selected -->
      <div v-if="selectedFile" class="flex flex-col h-full overflow-hidden">
        <!-- Tabs for Inspector (Details vs AI Prompts) -->
        <div class="tabs tabs-boxed bg-base-300/60 p-1 m-3 rounded-lg shrink-0 flex gap-1">
          <button
            class="tab tab-sm flex-1 text-xs py-1.5 transition-all duration-150 font-semibold rounded"
            :class="inspectorTab === 'info' ? 'bg-primary text-primary-content shadow-md' : 'text-base-content/50'"
            @click="inspectorTab = 'info'"
          >
            Details
          </button>
          <button
            class="tab tab-sm flex-1 text-xs py-1.5 transition-all duration-150 font-semibold rounded"
            :class="inspectorTab === 'ai' ? 'bg-primary text-primary-content shadow-md' : 'text-base-content/50'"
            @click="inspectorTab = 'ai'"
          >
            AI Prompts
          </button>
        </div>

        <!-- Tab contents -->
        <div class="flex-1 overflow-y-auto custom-scrollbar px-4 pb-4 space-y-4">
          <!-- A. Details Tab -->
          <div v-if="inspectorTab === 'info'" class="space-y-4">
            <!-- Thumbnail preview -->
            <div class="w-full aspect-square rounded-box bg-base-300 border border-base-200/50 overflow-hidden flex items-center justify-center shadow-lg relative group">
              <img
                v-if="isImage(selectedFile)"
                :src="getFileAssetUrl(selectedFile.path)"
                class="max-w-full max-h-full object-contain"
              />
              <video
                v-else-if="isVideo(selectedFile)"
                :src="getFileAssetUrl(selectedFile.path)"
                class="max-w-full max-h-full object-contain"
                muted
                autoplay
                loop
              ></video>
              <div v-else class="text-4xl text-base-content/30">📄</div>
            </div>

            <!-- Name (Editable inline) -->
            <div class="space-y-1">
              <label class="text-[10px] text-base-content/40 font-bold uppercase tracking-wider">File Name</label>
              <div class="flex items-center gap-1.5">
                <input
                  type="text"
                  v-model="renamingState.name"
                  class="input input-bordered input-sm flex-1 font-medium text-xs bg-base-300/40 border-base-200/40"
                  @keydown.enter="saveFileName"
                />
                <button
                  v-if="renamingState.name !== selectedFile.name"
                  class="btn btn-primary btn-sm h-8 min-h-[32px] px-3 font-semibold text-xs"
                  @click="saveFileName"
                >
                  Save
                </button>
              </div>
            </div>

            <!-- Size / Resolution -->
            <div class="grid grid-cols-2 gap-2 text-xs">
              <div class="p-2.5 rounded-lg bg-base-300/40 border border-base-200/20">
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">Size</span>
                <span class="font-semibold text-base-content/80 font-mono mt-0.5 block">{{ formatBytes(selectedFile.size) }}</span>
              </div>
              <div class="p-2.5 rounded-lg bg-base-300/40 border border-base-200/20">
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">Resolution</span>
                <span class="font-semibold text-base-content/80 font-mono mt-0.5 block">
                  {{ selectedFile.resolution ? `${selectedFile.resolution.width}×${selectedFile.resolution.height}` : '—' }}
                </span>
              </div>
            </div>

            <!-- Full Details list -->
            <div class="space-y-3 text-xs border-t border-base-100/20 pt-3">
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">Format</span>
                <span class="font-medium text-base-content/80 mt-0.5 block">{{ selectedFile.extension?.toUpperCase() || 'Unknown' }}</span>
              </div>
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">Modified</span>
                <span class="font-medium text-base-content/80 mt-0.5 block">{{ formatDate(selectedFile.modified) }}</span>
              </div>
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider text-ellipsis overflow-hidden">Full Path</span>
                <span class="font-mono text-[10px] text-base-content/70 mt-0.5 block break-all leading-normal">{{ selectedFile.path }}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex flex-col gap-2 pt-2 border-t border-base-100/20">
              <button class="btn btn-ghost btn-sm text-xs font-semibold justify-start hover:bg-base-100/40" @click="revealInExplorer(selectedFile.path)">
                📂 Show in System Explorer
              </button>
              <button v-if="isImage(selectedFile)" class="btn btn-ghost btn-sm text-xs font-semibold justify-start hover:bg-base-100/40" @click="openCrop(selectedFile)">
                ✂️ Quick Crop Image (K)
              </button>
              <button class="btn btn-ghost btn-sm text-xs font-semibold text-error justify-start hover:bg-error/10" @click="deleteSingleFile(selectedFile)">
                🗑️ Move to Trash
              </button>
            </div>
          </div>

          <!-- B. AI Prompts Tab -->
          <div v-else-if="inspectorTab === 'ai'">
            <PromptViewer :filePath="selectedFile.path" />
          </div>
        </div>
      </div>

      <!-- Multiple files selected -->
      <div v-else-if="galleryStore.selectedIds.length > 1" class="flex flex-col h-full items-center justify-center p-6 text-center text-base-content/50 space-y-4">
        <span class="text-5xl">📦</span>
        <div>
          <h4 class="font-bold text-base-content">{{ galleryStore.selectedIds.length }} items selected</h4>
          <p class="text-xs mt-1">Total size: {{ formatBytes(totalSelectedSize) }}</p>
        </div>

        <div class="flex flex-col gap-2 w-full pt-4">
          <button class="btn btn-primary btn-sm text-xs" @click="openCompare">
            ⚖️ Compare Selected Side-by-Side
          </button>
          <button class="btn btn-secondary btn-sm text-xs" @click="batchOperationsVisible = true">
            🛠️ Batch Process Selected
          </button>
          <button class="btn btn-ghost btn-sm text-xs text-error hover:bg-error/10" @click="deleteMultipleSelected">
            🗑️ Move Selected to Trash
          </button>
        </div>
      </div>

      <!-- Empty state (no files selected) -->
      <div v-else class="flex flex-col h-full items-center justify-center p-6 text-center text-base-content/30 space-y-3">
        <span class="text-4xl">ℹ️</span>
        <div>
          <h4 class="font-semibold text-sm text-base-content/50">No selection</h4>
          <p class="text-[11px] mt-1">Select a file to inspect details and prompt parameters.</p>
        </div>
      </div>
    </aside>

    <!-- Quick Look Overlay (Space/Double Click) -->
    <QuickLook
      :visible="quickLookVisible"
      :files="filteredFiles"
      :initial-index="quickLookIndex"
      @update:visible="quickLookVisible = $event"
    />

    <!-- Compare View Overlay -->
    <CompareView
      v-if="compareVisible"
      :files="compareFiles"
      @close="compareVisible = false"
    />

    <!-- Quick Crop Overlay -->
    <QuickCrop
      v-if="cropVisible"
      :file="cropFile"
      @close="cropVisible = false"
      @saved="onCropSaved"
    />

    <!-- Batch Operations Overlay -->
    <BatchOperations
      :visible="batchOperationsVisible"
      :selectedFiles="galleryStore.selectedIds"
      @close="batchOperationsVisible = false"
      @success="onBatchComplete"
    />
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNavigationStore } from '@/modules/navigation/store'
import { useGalleryStore } from '@/modules/gallery/store'
import { useConfigStore } from '@/stores/configStore'

// SVG Icons
import {
  IconFolders,
  IconTrash,
  IconSettings,
  IconNewFolder,
  IconRefresh,
} from '@/common/icons'

// Overlays and modules
import ExplorerTree from '@/modules/navigation/components/ExplorerTree.vue'
import FilterBar from '@/modules/gallery/components/FilterBar.vue'
import GalleryGrid from '@/modules/gallery/components/GalleryGrid.vue'
import QuickLook from '@/modules/viewer/components/QuickLook.vue'
import PromptViewer from '@/modules/viewer/components/PromptViewer.vue'
import CompareView from '@/modules/viewer/components/CompareView.vue'
import QuickCrop from '@/modules/viewer/components/QuickCrop.vue'
import BatchOperations from '@/modules/operations/components/BatchOperations.vue'

const navigationStore = useNavigationStore()
const galleryStore = useGalleryStore()
const configStore = useConfigStore()

const activeTab = ref('explorer') // 'explorer' | 'trash'
const showSidebar = ref(true)
const inspectorTab = ref('info') // 'info' | 'ai'

// Splitters dragging state
const isDraggingLeftSplitter = ref(false)
const isDraggingRightSplitter = ref(false)

// Trash list state
const trashItems = ref([])
const trashCount = computed(() => trashItems.value.length)

// Action visibility
const compareVisible = ref(false)
const compareFiles = ref([])
const cropVisible = ref(false)
const cropFile = ref(null)
const batchOperationsVisible = ref(false)

// Quick Look state
const quickLookVisible = ref(false)
const quickLookIndex = ref(0)
const filteredFiles = computed(() => galleryStore.displayedFiles)

// Selected details
const selectedFile = computed(() => {
  if (galleryStore.selectedIds.length === 1) {
    const path = galleryStore.selectedIds[0]
    return galleryStore.displayedFiles.find((f) => f.path === path) || null
  }
  return null
})

// Rename state
const renamingState = reactive({
  name: '',
})

watch(selectedFile, (newFile) => {
  if (newFile) {
    renamingState.name = newFile.name
  } else {
    renamingState.name = ''
  }
})

// Calculate sizes
const totalSelectedSize = computed(() => {
  return galleryStore.selectedIds.reduce((sum, path) => {
    const file = galleryStore.displayedFiles.find((f) => f.path === path)
    return sum + (file?.size || 0)
  }, 0)
})

// Fetch trash items periodically or when activeTab is 'trash'
async function fetchTrash() {
  try {
    trashItems.value = await invoke('get_trash_contents')
  } catch (err) {
    console.error('Failed to load trash:', err)
  }
}

watch(activeTab, (val) => {
  if (val === 'trash') {
    fetchTrash()
  }
})

// Breadcrumbs builder
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

// Navigation methods
async function navigateTo(path) {
  if (!path) return
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

async function refreshData() {
  await navigationStore.refresh()
  galleryStore.setFiles(navigationStore.folders)
  if (activeTab.value === 'trash') {
    fetchTrash()
  }
}

function toggleInspector() {
  configStore.rightPanel.show = !configStore.rightPanel.show
}

// Dialog functions
function openSettings() {
  invoke('get_app_config').then(() => {
    // Open settings window from rust
    invoke('select_folder', { albumId: 0, folderPath: '' }).catch(() => {}) // triggers settings view if window matches
  })
  // Fallback trigger window:
  emit('app-open-preferences')
}

// Explorer File actions
async function createFolder() {
  const name = prompt('Enter folder name:')
  if (name && navigationStore.currentPath) {
    try {
      await invoke('create_folder', { path: navigationStore.currentPath, folderName: name })
      await refreshData()
    } catch (e) {
      console.error(e)
    }
  }
}

async function saveFileName() {
  if (!selectedFile.value || !renamingState.name || renamingState.name === selectedFile.value.name) return
  const oldPath = selectedFile.value.path
  const index = oldPath.lastIndexOf('\\')
  const dir = index >= 0 ? oldPath.substring(0, index + 1) : ''
  const newPath = dir + renamingState.name

  try {
    await invoke('cross_move', { src: oldPath, dest: newPath })
    await refreshData()
    galleryStore.selectedIds = [newPath]
  } catch (err) {
    alert(typeof err === 'string' ? err : 'Rename failed')
  }
}

function revealInExplorer(path) {
  invoke('open_in_explorer', { path })
}

// Single delete
async function deleteSingleFile(file) {
  if (confirm(`Move ${file.name} to trash?`)) {
    try {
      await invoke('move_to_trash', { paths: [file.path] })
      await refreshData()
      galleryStore.clearSelection()
    } catch (err) {
      alert('Delete failed')
    }
  }
}

// Multiple deletes
async function deleteMultipleSelected() {
  if (confirm(`Move ${galleryStore.selectedIds.length} items to trash?`)) {
    try {
      await invoke('move_to_trash', { paths: galleryStore.selectedIds })
      await refreshData()
      galleryStore.clearSelection()
    } catch (err) {
      alert('Delete failed')
    }
  }
}

// Compare
function openCompare() {
  if (galleryStore.selectedIds.length < 2) return
  compareFiles.value = galleryStore.selectedIds.map(path => {
    return galleryStore.displayedFiles.find(f => f.path === path)
  }).filter(Boolean)
  compareVisible.value = true
}

// Crop
function openCrop(file) {
  cropFile.value = file
  cropVisible.value = true
}

function onCropSaved() {
  refreshData()
}

function onBatchComplete() {
  refreshData()
  batchOperationsVisible.value = false
}

// Trash Bin actions
async function restoreTrashFile(trashPath) {
  try {
    await invoke('restore_from_trash', { trashPaths: [trashPath] })
    await refreshData()
  } catch (err) {
    alert('Failed to restore file')
  }
}

async function deleteTrashFilePermanently(item) {
  if (confirm('Permanently delete this file from disk? This cannot be undone.')) {
    try {
      // Since it is in trash, we can delete the file from the trash folder permanently
      await invoke('delete_file_system', { path: item.trashPath })
      // Remove meta file too
      const meta = item.trashPath.replace(/\.[^/.]+$/, "") + ".meta.json"
      await invoke('delete_file_system', { path: meta }).catch(() => {})
      await refreshData()
    } catch (err) {
      alert('Failed to delete file')
    }
  }
}

async function clearTrash() {
  if (confirm('Empty trash bin permanently? This deletes all files inside the trash bin.')) {
    try {
      await invoke('empty_trash')
      await refreshData()
    } catch (err) {
      alert('Failed to empty trash')
    }
  }
}

// Drag resizers
function startDraggingLeftSplitter(e) {
  isDraggingLeftSplitter.value = true
  document.addEventListener('mousemove', dragLeftSplitter)
  document.addEventListener('mouseup', stopDraggingLeftSplitter)
}

function dragLeftSplitter(e) {
  if (!isDraggingLeftSplitter.value) return
  configStore.leftPanel.width = Math.max(160, Math.min(e.clientX - 64, window.innerWidth / 2))
}

function stopDraggingLeftSplitter() {
  isDraggingLeftSplitter.value = false
  document.removeEventListener('mousemove', dragLeftSplitter)
  document.removeEventListener('mouseup', stopDraggingLeftSplitter)
}

function startDraggingRightSplitter(e) {
  isDraggingRightSplitter.value = true
  document.addEventListener('mousemove', dragRightSplitter)
  document.addEventListener('mouseup', stopDraggingRightSplitter)
}

function dragRightSplitter(e) {
  if (!isDraggingRightSplitter.value) return
  configStore.rightPanel.width = Math.max(220, Math.min(window.innerWidth - e.clientX, window.innerWidth / 2))
}

function stopDraggingRightSplitter() {
  isDraggingRightSplitter.value = false
  document.removeEventListener('mousemove', dragRightSplitter)
  document.removeEventListener('mouseup', stopDraggingRightSplitter)
}

// Helper formatting
function formatBytes(bytes) {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`
}

function formatDate(dateStr) {
  if (!dateStr) return '—'
  try {
    const d = new Date(dateStr)
    return d.toLocaleString()
  } catch {
    return dateStr
  }
}

function getFileName(path) {
  return path.split('\\').pop() || path.split('/').pop() || path
}

function getFileAssetUrl(path) {
  return `asset://localhost/${encodeURI(path)}`
}

function isImage(file) {
  const ext = file.extension?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'avif', 'jxl', 'svg', 'ico'].includes(ext)
}

function isVideo(file) {
  const ext = file.extension?.toLowerCase()
  return ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mpeg', '3gp'].includes(ext)
}

// Quick Look overlays
function openQuickLook(file) {
  const index = filteredFiles.value.findIndex(f => f.path === file.path)
  if (index >= 0) {
    quickLookIndex.value = index
    quickLookVisible.value = true
  }
}

// Keyboard shortcuts global handler
function handleKeyDown(e) {
  // Crop via K
  if (e.key === 'k' && selectedFile.value && isImage(selectedFile.value)) {
    e.preventDefault()
    openCrop(selectedFile.value)
  }
  // Compare via C
  if (e.key === 'c' && galleryStore.selectedIds.length >= 2) {
    e.preventDefault()
    openCompare()
  }
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeyDown)
  await navigationStore.loadDrives()
  await fetchTrash()

  // Load default path (Home or C:)
  const homeDrive = navigationStore.drives.find(d => d.path.startsWith('C:'))
  if (homeDrive) {
    await navigateTo(homeDrive.path)
  } else if (navigationStore.drives.length > 0) {
    await navigateTo(navigationStore.drives[0].path)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style>
/* Global Premium Styles for scrollbars and inputs */
.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--color-base-content) 15%, transparent);
  border-radius: 9999px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--color-base-content) 30%, transparent);
}

.animate-hover-spin:hover {
  transform: rotate(30deg);
}
</style>