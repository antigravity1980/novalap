<template>
  <div class="main-layout flex h-screen w-screen overflow-hidden bg-base-300 text-base-content select-none font-sans">
    <!-- Navigation & Explorer sidebar panel (Unified Left Sidebar) -->
    <aside
      v-if="showSidebar"
      class="sidebar-panel border-r border-neutral/30 bg-base-200 flex flex-col overflow-hidden transition-all duration-300"
      :style="{ width: configStore.leftPanel.width + 'px' }"
    >
      <!-- Windows 11 Sidebar navigation list -->
      <div class="flex-1 overflow-y-auto custom-scrollbar py-3 px-2 space-y-4">
        <!-- Quick Access Section -->
        <div class="space-y-1">
          <div class="px-3 py-1 text-[10px] font-bold text-base-content/40 uppercase tracking-widest">
            {{ $t('explorer.quick_access') }}
          </div>
          <!-- Home item -->
          <div
            class="flex items-center gap-2.5 px-3 py-2 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150 relative"
            :class="activeTab === 'explorer' && !navigationStore.currentPath ? 'bg-primary/15 text-primary font-bold active-nav-item' : 'text-base-content/75 hover:bg-base-100/50 hover:text-base-content'"
            @click="navigateToHome"
          >
            <IconHome class="w-4 h-4 shrink-0" />
            <span>{{ $t('explorer.home') }}</span>
          </div>
          <!-- Recycle Bin item -->
          <div
            class="flex items-center gap-2.5 px-3 py-2 cursor-pointer rounded-lg text-xs font-semibold transition-all duration-150 relative"
            :class="activeTab === 'trash' ? 'bg-primary/15 text-primary font-bold active-nav-item' : 'text-base-content/75 hover:bg-base-100/50 hover:text-base-content'"
            @click="activeTab = 'trash'"
          >
            <IconTrash class="w-4 h-4 shrink-0" />
            <span class="flex-1">{{ $t('explorer.recycle_bin') }}</span>
            <span v-if="trashCount > 0" class="badge badge-error badge-sm scale-75">{{ trashCount }}</span>
          </div>
        </div>

        <!-- This PC / Drives Section -->
        <div class="space-y-1">
          <div class="px-3 py-1 text-[10px] font-bold text-base-content/40 uppercase tracking-widest">
            {{ $t('explorer.this_pc') }}
          </div>
          <!-- Render the Drive list and ExplorerTree folders under it -->
          <ExplorerTree />
        </div>
      </div>

      <!-- Sidebar Bottom Settings -->
      <div class="p-2 border-t border-neutral/20 shrink-0 bg-base-200/50 flex items-center justify-between">
        <button
          class="btn btn-ghost btn-sm w-full gap-2.5 justify-start font-semibold text-xs text-base-content/70 hover:text-base-content"
          :title="$t('explorer.settings')"
          @click="openSettings"
        >
          <IconSettings class="w-4 h-4 shrink-0" />
          <span>{{ $t('explorer.settings') }}</span>
        </button>
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
    <main class="flex-1 flex flex-col overflow-hidden bg-base-100">
      <!-- Windows 11 Tabs at the top -->
      <div 
        class="h-10 bg-base-200 border-b border-neutral/25 flex items-center pl-4 pr-1 justify-between shrink-0 select-none"
        data-tauri-drag-region
      >
        <div class="flex items-center gap-1 overflow-hidden h-full pt-1.5" @mousedown.stop>
          <!-- Active Tab -->
          <div class="flex items-center gap-2 px-4 h-full bg-base-100 rounded-t-lg border-t border-x border-neutral/25 text-xs font-semibold text-base-content/85 select-none relative shrink-0">
            <IconTrash v-if="activeTab === 'trash'" class="w-3.5 h-3.5 text-primary" />
            <IconFolders v-else-if="navigationStore.currentPath" class="w-3.5 h-3.5 text-primary" />
            <IconHome v-else class="w-3.5 h-3.5 text-primary" />
            <span>{{ activeTab === 'trash' ? $t('explorer.recycle_bin') : (navigationStore.currentPath ? getFileName(navigationStore.currentPath) : $t('explorer.home')) }}</span>
            <button class="hover:bg-base-200 rounded-full w-4 h-4 flex items-center justify-center text-[10px] opacity-60 hover:opacity-100 ml-1">✕</button>
          </div>
          <!-- Add Tab button -->
          <button class="w-7 h-7 rounded-md hover:bg-base-100/60 flex items-center justify-center text-sm text-base-content/60 hover:text-base-content font-bold shrink-0">+</button>
        </div>

        <!-- Window Control Buttons (Minimize, Maximize, Close) -->
        <div class="flex items-center h-full shrink-0" @mousedown.stop>
          <button 
            @click="minimizeWindow" 
            :title="$t('msgbox.discard')" 
            class="w-11 h-full flex items-center justify-center text-base-content/75 hover:bg-base-100/40 hover:text-base-content transition-colors duration-100"
          >
            <svg class="w-2.5 h-2.5" viewBox="0 0 10 1" fill="none" stroke="currentColor" stroke-width="1.2">
              <line x1="0" y1="0.5" x2="10" y2="0.5" />
            </svg>
          </button>
          <button 
            @click="toggleMaximizeWindow" 
            :title="$t('image_editor.transform')" 
            class="w-11 h-full flex items-center justify-center text-base-content/75 hover:bg-base-100/40 hover:text-base-content transition-colors duration-100"
          >
            <svg v-if="!isMaximized" class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="0.5" y="0.5" width="9" height="9" />
            </svg>
            <svg v-else class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="1.5" y="1.5" width="7" height="7" />
              <path d="M1.5 1.5V0.5H8.5V7.5H7.5" />
            </svg>
          </button>
          <button 
            @click="closeWindow" 
            :title="$t('msgbox.close')" 
            class="w-11 h-full flex items-center justify-center text-base-content/75 hover:bg-red-500 hover:text-white transition-colors duration-100"
          >
            <svg class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
              <path d="M0.5 0.5 L9.5 9.5 M9.5 0.5 L0.5 9.5" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Navigation address bar & Controls -->
      <header class="h-12 flex items-center justify-between px-6 border-b border-neutral/25 bg-base-200 shrink-0 z-10 gap-4">
        <!-- Navigation Arrows & Controls -->
        <div class="flex items-center gap-2.5 shrink-0">
          <button
            class="btn btn-ghost btn-xs btn-circle text-base-content/75 hover:text-base-content hover:bg-base-100/40 flex items-center justify-center"
            @click="goBack"
            :disabled="!navigationStore.canGoBack"
            :title="$t('welcome.drag_title')"
          >
            <IconLeft class="w-3.5 h-3.5 shrink-0" />
          </button>
          <button
            class="btn btn-ghost btn-xs btn-circle text-base-content/75 hover:text-base-content hover:bg-base-100/40 flex items-center justify-center"
            @click="goForward"
            :disabled="!navigationStore.canGoForward"
            :title="$t('welcome.drag_title')"
          >
            <IconRight class="w-3.5 h-3.5 shrink-0" />
          </button>
          <button
            class="btn btn-ghost btn-xs btn-circle text-base-content/75 hover:text-base-content hover:bg-base-100/40 flex items-center justify-center"
            @click="goUp"
            :disabled="!navigationStore.currentPath"
            :title="$t('welcome.drag_title')"
          >
            <IconArrowUp class="w-3.5 h-3.5 shrink-0" />
          </button>
          <button
            class="btn btn-ghost btn-xs btn-circle text-base-content/75 hover:text-base-content hover:bg-base-100/40 flex items-center justify-center"
            @click="refreshData"
            :title="$t('toolbar.tooltip.refresh')"
          >
            <IconRefresh class="w-3.5 h-3.5 shrink-0 animate-hover-spin" />
          </button>
        </div>

        <!-- Address Bar (Editable path) -->
        <div class="flex-1 flex items-center border border-neutral/30 rounded bg-base-100 px-3 py-1 text-xs select-none max-w-3xl h-8">
          <span class="mr-2 opacity-65 text-sm">💻</span>
          
          <div v-if="!isEditingPath" class="flex-1 flex items-center overflow-x-auto no-scrollbar whitespace-nowrap gap-1 cursor-text h-full" @click="startEditingPath">
            <div class="flex items-center gap-1 text-base-content/85">
              <span class="hover:text-primary transition-colors cursor-pointer" @click.stop="navigateToHome">{{ $t('explorer.this_pc') }}</span>
              <span class="text-base-content/40 text-[9px] font-bold">▸</span>
            </div>
            <div v-for="(crumb, i) in breadcrumbs" :key="crumb.path" class="flex items-center gap-1 text-base-content/85">
              <span class="hover:text-primary transition-colors cursor-pointer" @click.stop="navigateTo(crumb.path)">{{ crumb.name }}</span>
              <span class="text-base-content/40 text-[9px] font-bold" v-if="i < breadcrumbs.length - 1">▸</span>
            </div>
          </div>

          <input
            v-else
            ref="pathInputRef"
            type="text"
            v-model="editablePath"
            class="flex-1 bg-transparent border-0 outline-none text-xs text-base-content/90 w-full"
            @keydown.enter="commitPathEdit"
            @blur="commitPathEdit"
          />
        </div>

        <!-- Search Bar (Far Right) -->
        <div class="relative w-56 shrink-0 flex items-center">
          <input
            type="text"
            v-model="searchQuery"
            :placeholder="$t('explorer.search_placeholder')"
            class="input input-bordered input-sm w-full pl-8 text-xs bg-base-100 border-neutral/30"
          />
          <IconSearch class="absolute left-2.5 text-base-content/40 w-3.5 h-3.5" />
        </div>
      </header>

      <!-- Command Bar -->
      <div class="h-12 border-b border-neutral/25 bg-base-200/50 flex items-center justify-between shrink-0 select-none overflow-x-auto no-scrollbar">
        <div class="flex items-center gap-1 px-3 shrink-0">
          <!-- New Folder -->
          <button
            v-if="activeTab === 'explorer' && navigationStore.currentPath"
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            @click="createFolder"
          >
            <IconNewFolder class="w-4 h-4 text-primary" />
            <span>{{ $t('explorer.new_folder') }}</span>
          </button>

          <div v-if="activeTab === 'explorer' && navigationStore.currentPath" class="divider divider-horizontal h-4 mx-1.5 self-center"></div>

          <!-- Rename -->
          <button
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            :disabled="!selectedFile"
            @click="focusRenameInput"
            :title="$t('explorer.rename')"
          >
            <IconRename class="w-4 h-4" />
            <span>{{ $t('explorer.rename') }}</span>
          </button>

          <!-- Delete -->
          <button
            class="win11-btn text-xs font-semibold text-error hover:bg-error/10 flex items-center gap-1.5"
            :disabled="galleryStore.selectedIds.length === 0"
            @click="deleteMultipleSelected"
            :title="$t('explorer.delete')"
          >
            <IconTrash class="w-4 h-4 text-error" />
            <span>{{ $t('explorer.delete') }}</span>
          </button>

          <!-- Compare -->
          <button
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            :disabled="galleryStore.selectedIds.length < 2"
            @click="openCompare"
            :title="$t('explorer.compare')"
          >
            <IconSplitOn class="w-4 h-4" />
            <span>{{ $t('explorer.compare') }}</span>
          </button>

          <!-- Batch -->
          <button
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            :disabled="galleryStore.selectedIds.length === 0"
            @click="batchOperationsVisible = true"
            :title="$t('explorer.batch')"
          >
            <IconAdjustments class="w-4 h-4" />
            <span>{{ $t('explorer.batch') }}</span>
          </button>

          <div class="divider divider-horizontal h-4 mx-1.5 self-center"></div>

          <!-- Sort Dropdown -->
          <div class="dropdown">
            <label tabindex="0" class="win11-btn flex items-center gap-1.5">
              <IconArrowUpDown class="w-4 h-4 text-primary" />
              <span>{{ $t('explorer.sort') }}</span>
              <span class="opacity-50">▾</span>
            </label>
            <ul tabindex="0" class="dropdown-content menu p-1.5 shadow-2xl bg-base-300 border border-neutral/30 rounded-lg w-44 z-30 text-xs mt-1">
              <li><a :class="{ 'active': galleryStore.sortBy === 'name' }" @click="galleryStore.sortBy = 'name'; onSortChange()">{{ $t('album.edit.name') }}</a></li>
              <li><a :class="{ 'active': galleryStore.sortBy === 'size' }" @click="galleryStore.sortBy = 'size'; onSortChange()">{{ $t('explorer.size') }}</a></li>
              <li><a :class="{ 'active': galleryStore.sortBy === 'date' }" @click="galleryStore.sortBy = 'date'; onSortChange()">{{ $t('calendar.title') }}</a></li>
              <li><a :class="{ 'active': galleryStore.sortBy === 'resolution' }" @click="galleryStore.sortBy = 'resolution'; onSortChange()">{{ $t('explorer.resolution') }}</a></li>
              <li><a :class="{ 'active': galleryStore.sortBy === 'ai_source' }" @click="galleryStore.sortBy = 'ai_source'; onSortChange()">{{ $t('explorer.ai_source') }}</a></li>
              <div class="divider my-1"></div>
              <li><a @click="toggleSortOrder">{{ $t('toolbar.tooltip.sort') }}: {{ galleryStore.sortOrder === 'asc' ? $t('toolbar.filter.sort_order_options[0]') + ' (↑)' : $t('toolbar.filter.sort_order_options[1]') + ' (↓)' }}</a></li>
            </ul>
          </div>

          <!-- Filter Dropdown -->
          <div class="dropdown">
            <label tabindex="0" class="win11-btn flex items-center gap-1.5">
              <IconSearch class="w-4 h-4" />
              <span>{{ $t('explorer.filter') }}</span>
              <span class="opacity-50">▾</span>
            </label>
            <ul tabindex="0" class="dropdown-content menu p-1.5 shadow-2xl bg-base-300 border border-neutral/30 rounded-lg w-48 z-30 text-xs mt-1 max-h-96 overflow-y-auto custom-scrollbar">
              <div class="px-2 py-1 text-[9px] font-bold text-base-content/40 uppercase">{{ $t('explorer.format') }}</div>
              <li><a :class="{ 'active': galleryStore.filters.format === '' }" @click="galleryStore.filters.format = ''; onFilterChange()">{{ $t('explorer.all_formats') }}</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'png' }" @click="galleryStore.filters.format = 'png'; onFilterChange()">PNG</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'jpg' || galleryStore.filters.format === 'jpeg' }" @click="galleryStore.filters.format = 'jpg'; onFilterChange()">JPEG</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'webp' }" @click="galleryStore.filters.format = 'webp'; onFilterChange()">WebP</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'gif' }" @click="galleryStore.filters.format = 'gif'; onFilterChange()">GIF</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'mp4' }" @click="galleryStore.filters.format = 'mp4'; onFilterChange()">MP4</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'mkv' }" @click="galleryStore.filters.format = 'mkv'; onFilterChange()">MKV</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'webm' }" @click="galleryStore.filters.format = 'webm'; onFilterChange()">WebM</a></li>
              <li><a :class="{ 'active': galleryStore.filters.format === 'mov' }" @click="galleryStore.filters.format = 'mov'; onFilterChange()">MOV</a></li>

              <div class="divider my-1"></div>
              <div class="px-2 py-1 text-[9px] font-bold text-base-content/40 uppercase">{{ $t('explorer.ai_source') }}</div>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === '' }" @click="galleryStore.filters.aiSource = ''; onFilterChange()">{{ $t('explorer.all_sources') }}</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'ComfyUI' }" @click="galleryStore.filters.aiSource = 'ComfyUI'; onFilterChange()">ComfyUI</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Midjourney' }" @click="galleryStore.filters.aiSource = 'Midjourney'; onFilterChange()">Midjourney</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Stable Diffusion' }" @click="galleryStore.filters.aiSource = 'Stable Diffusion'; onFilterChange()">Stable Diffusion</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Nano Banana' }" @click="galleryStore.filters.aiSource = 'Nano Banana'; onFilterChange()">Nano Banana</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'GPT Images' }" @click="galleryStore.filters.aiSource = 'GPT Images'; onFilterChange()">GPT Images</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Grok Image' }" @click="galleryStore.filters.aiSource = 'Grok Image'; onFilterChange()">Grok Image</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'DALL-E' }" @click="galleryStore.filters.aiSource = 'DALL-E'; onFilterChange()">DALL-E</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Krita AI' }" @click="galleryStore.filters.aiSource = 'Krita AI'; onFilterChange()">Krita AI</a></li>
              <li><a :class="{ 'active': galleryStore.filters.aiSource === 'Unknown' }" @click="galleryStore.filters.aiSource = 'Unknown'; onFilterChange()">{{ $t('explorer.unknown_source') }}</a></li>
            </ul>
          </div>
        </div>

        <!-- Theme Toggle and Details pane toggle -->
        <div class="flex items-center gap-1.5 px-3 shrink-0">
          <!-- Theme Toggle -->
          <button
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            @click="toggleTheme"
            title="Toggle light / dark theme"
          >
            <span v-if="configStore.settings.appearance === 0" class="flex items-center gap-1.5">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
              </svg>
              <span>{{ $t('explorer.dark_theme') }}</span>
            </span>
            <span v-else class="flex items-center gap-1.5">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-amber-500 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
              </svg>
              <span>{{ $t('explorer.light_theme') }}</span>
            </span>
          </button>

          <div class="divider divider-horizontal h-4 mx-1.5 self-center"></div>

          <!-- Info toggle -->
          <button
            class="win11-btn text-xs font-semibold flex items-center gap-1.5"
            :class="{ 'bg-primary/10 text-primary font-bold': configStore.rightPanel.show }"
            @click="toggleInspector"
            :title="$t('explorer.details')"
          >
            <IconInformation class="w-4 h-4 text-primary" />
            <span>{{ $t('explorer.details') }}</span>
          </button>
        </div>
      </div>

      <!-- Main gallery area -->
      <div class="flex-1 overflow-hidden relative bg-base-100">
        <!-- Recycle Bin contents list overlay -->
        <div v-if="activeTab === 'trash'" class="absolute inset-0 z-10 bg-base-100 overflow-y-auto p-6 space-y-4">
          <div class="flex items-center justify-between border-b border-neutral/20 pb-4">
            <div>
              <h2 class="text-lg font-bold text-base-content">{{ $t('explorer.recycle_bin') }}</h2>
              <p class="text-xs text-base-content/50 mt-1">{{ $t('explorer.recycle_bin_hint') }}</p>
            </div>
            <button
              class="btn btn-error btn-sm font-semibold rounded-md"
              :disabled="trashItems.length === 0"
              @click="clearTrash"
            >
              {{ $t('explorer.empty_trash') }}
            </button>
          </div>

          <div v-if="trashItems.length === 0" class="flex flex-col items-center justify-center py-24 text-center text-base-content/30 space-y-3">
            <span class="text-5xl">🗑️</span>
            <span class="text-sm font-medium">{{ $t('explorer.recycle_bin_empty') }}</span>
          </div>

          <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <div
              v-for="item in trashItems"
              :key="item.trashPath"
              class="p-3.5 rounded-lg bg-base-200 border border-neutral/25 flex flex-col justify-between hover:bg-neutral/10 transition-all duration-150 relative group"
            >
              <div class="pr-6">
                <p class="text-xs font-semibold text-base-content/85 truncate" :title="getFileName(item.originalPath)">
                  {{ getFileName(item.originalPath) }}
                </p>
                <p class="text-[10px] text-base-content/40 truncate mt-0.5" :title="item.originalPath">
                  {{ item.originalPath }}
                </p>
              </div>
              <div class="flex items-center justify-between mt-3 text-[10px] text-base-content/50 border-t border-neutral/15 pt-2">
                <span>{{ formatBytes(item.size) }}</span>
                <div class="flex gap-2">
                  <button class="btn btn-primary btn-xs px-2.5 h-6 min-h-0 rounded text-[10px]" @click="restoreTrashFile(item.trashPath)">
                    {{ $t('explorer.restore') }}
                  </button>
                  <button class="btn btn-ghost btn-xs px-2.5 h-6 min-h-0 rounded text-[10px] text-error hover:bg-error/10" @click="deleteTrashFilePermanently(item)">
                    {{ $t('explorer.delete_permanently') }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <GalleryGrid v-else @open-quick-look="openQuickLook" />
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
      class="inspector-panel border-l border-neutral/30 bg-base-200 flex flex-col overflow-hidden shrink-0 transition-all duration-300 z-10"
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
            {{ $t('explorer.details_tab') }}
          </button>
          <button
            class="tab tab-sm flex-1 text-xs py-1.5 transition-all duration-150 font-semibold rounded"
            :class="inspectorTab === 'ai' ? 'bg-primary text-primary-content shadow-md' : 'text-base-content/50'"
            @click="inspectorTab = 'ai'"
          >
            {{ $t('explorer.ai_prompts_tab') }}
          </button>
        </div>

        <!-- Tab contents -->
        <div class="flex-1 overflow-y-auto custom-scrollbar px-4 pb-4 space-y-4">
          <!-- A. Details Tab -->
          <div v-if="inspectorTab === 'info'" class="space-y-4">
            <!-- Thumbnail preview -->
            <div class="w-full aspect-square rounded-box bg-base-300 border border-neutral/20 overflow-hidden flex items-center justify-center shadow-lg relative group">
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
              <label class="text-[10px] text-base-content/40 font-bold uppercase tracking-wider">{{ $t('explorer.file_name') }}</label>
              <div class="flex items-center gap-1.5">
                <input
                  id="rename-input-field"
                  ref="renameInputRef"
                  type="text"
                  v-model="renamingState.name"
                  class="input input-bordered input-sm flex-1 font-medium text-xs bg-base-100 border-neutral/35"
                  @keydown.enter="saveFileName"
                />
                <button
                  v-if="renamingState.name !== selectedFile.name"
                  class="btn btn-primary btn-sm h-8 min-h-[32px] px-3 font-semibold text-xs rounded-md"
                  @click="saveFileName"
                >
                  {{ $t('explorer.save') }}
                </button>
              </div>
            </div>

            <!-- Size / Resolution -->
            <div class="grid grid-cols-2 gap-2 text-xs">
              <div class="p-2.5 rounded-lg bg-base-100 border border-neutral/20">
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">{{ $t('explorer.size') }}</span>
                <span class="font-semibold text-base-content/85 font-mono mt-0.5 block">{{ formatBytes(selectedFile.size) }}</span>
              </div>
              <div class="p-2.5 rounded-lg bg-base-100 border border-neutral/20">
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">{{ $t('explorer.resolution') }}</span>
                <span class="font-semibold text-base-content/85 font-mono mt-0.5 block">
                  {{ selectedFile.resolution ? `${selectedFile.resolution.width}×${selectedFile.resolution.height}` : '—' }}
                </span>
              </div>
            </div>

            <!-- ComfyUI Workflow Detected Badge -->
            <div v-if="hasComfyWorkflow" class="p-2 bg-green-500/10 text-green-600 dark:text-green-400 border border-green-500/20 rounded-lg text-[10px] font-bold uppercase tracking-wider flex items-center justify-center gap-1.5 shadow-sm">
              <span>⚡ {{ $t('explorer.comfyui_workflow_detected') }}</span>
            </div>

            <!-- Full Details list -->
            <div class="space-y-3 text-xs border-t border-neutral/20 pt-3">
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">{{ $t('explorer.format') }}</span>
                <span class="font-medium text-base-content/85 mt-0.5 block">{{ selectedFile.extension?.toUpperCase() || 'Unknown' }}</span>
              </div>
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider">{{ $t('explorer.modified') }}</span>
                <span class="font-medium text-base-content/85 mt-0.5 block">{{ formatDate(selectedFile.modified) }}</span>
              </div>
              <div>
                <span class="block text-[9px] text-base-content/40 font-bold uppercase tracking-wider text-ellipsis overflow-hidden">{{ $t('explorer.full_path') }}</span>
                <span class="font-mono text-[10px] text-base-content/70 mt-0.5 block break-all leading-normal">{{ selectedFile.path }}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex flex-col gap-1.5 pt-2 border-t border-neutral/20">
              <button class="btn btn-ghost btn-sm text-xs font-semibold justify-start hover:bg-base-100/40 rounded-md" @click="revealInExplorer(selectedFile.path)">
                📂 {{ $t('explorer.show_in_system_explorer') }}
              </button>
              <button v-if="isImage(selectedFile)" class="btn btn-ghost btn-sm text-xs font-semibold justify-start hover:bg-base-100/40 rounded-md" @click="openCrop(selectedFile)">
                ✂️ {{ $t('explorer.quick_crop') }}
              </button>
              <button class="btn btn-ghost btn-sm text-xs font-semibold text-error justify-start hover:bg-error/10 rounded-md" @click="deleteSingleFile(selectedFile)">
                🗑️ {{ $t('explorer.move_to_trash') }}
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
          <h4 class="font-bold text-base-content">{{ $t('explorer.items_selected', { count: galleryStore.selectedIds.length }) }}</h4>
          <p class="text-xs mt-1">{{ $t('explorer.total_size', { size: formatBytes(totalSelectedSize) }) }}</p>
        </div>

        <div class="flex flex-col gap-2 w-full pt-4">
          <button class="btn btn-primary btn-sm text-xs rounded-md" @click="openCompare">
            ⚖️ {{ $t('explorer.compare_selected') }}
          </button>
          <button class="btn btn-secondary btn-sm text-xs rounded-md" @click="batchOperationsVisible = true">
            🛠️ {{ $t('explorer.batch_process_selected') }}
          </button>
          <button class="btn btn-ghost btn-sm text-xs text-error hover:bg-error/10 rounded-md" @click="deleteMultipleSelected">
            🗑️ {{ $t('explorer.move_selected_to_trash') }}
          </button>
        </div>
      </div>

      <!-- Empty state (no files selected) -->
      <div v-else class="flex flex-col h-full items-center justify-center p-6 text-center text-base-content/30 space-y-3">
        <span class="text-4xl">ℹ️</span>
        <div>
          <h4 class="font-semibold text-sm text-base-content/50">{{ $t('explorer.no_selection') }}</h4>
          <p class="text-[11px] mt-1">{{ $t('explorer.select_hint') }}</p>
        </div>
      </div>
    </aside>

    <!-- Quick Look Overlay (Space/Double Click) -->
    <QuickLook
      :visible="quickLookVisible"
      :files="filteredFiles"
      :initial-index="quickLookIndex"
      @update:visible="quickLookVisible = $event"
      @saved="refreshData"
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
import { ref, reactive, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'

const router = useRouter()
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useNavigationStore } from '@/modules/navigation/store'
import { useGalleryStore } from '@/modules/gallery/store'
import { useConfigStore } from '@/stores/configStore'
import { setTheme } from '@/common/utils'

// SVG Icons
import {
  IconFolders,
  IconTrash,
  IconSettings,
  IconNewFolder,
  IconRefresh,
  IconRename,
  IconSplitOn,
  IconAdjustments,
  IconArrowUpDown,
  IconSearch,
  IconInformation,
  IconHome,
  IconLeft,
  IconRight,
  IconArrowUp
} from '@/common/icons'

// Overlays and modules
import ExplorerTree from '@/modules/navigation/components/ExplorerTree.vue'
import GalleryGrid from '@/modules/gallery/components/GalleryGrid.vue'
import QuickLook from '@/modules/viewer/components/QuickLook.vue'
import PromptViewer from '@/modules/viewer/components/PromptViewer.vue'
import CompareView from '@/modules/viewer/components/CompareView.vue'
import QuickCrop from '@/modules/viewer/components/QuickCrop.vue'
import BatchOperations from '@/modules/operations/components/BatchOperations.vue'

const navigationStore = useNavigationStore()
const galleryStore = useGalleryStore()
const configStore = useConfigStore()

// Safe window control helper for browser dev server support
function getSafeWindow() {
  try {
    return getCurrentWindow();
  } catch (e) {
    return {
      minimize: () => console.log('Minimize window'),
      maximize: () => console.log('Maximize window'),
      unmaximize: () => console.log('Unmaximize window'),
      isMaximized: () => Promise.resolve(false),
      close: () => console.log('Close window'),
    };
  }
}
const appWindow = getSafeWindow()
const isMaximized = ref(false)

const minimizeWindow = () => {
  appWindow.minimize()
}

const toggleMaximizeWindow = async () => {
  try {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      isMaximized.value = false
      await appWindow.unmaximize()
    } else {
      isMaximized.value = true
      await appWindow.maximize()
    }
  } catch (e) {
    isMaximized.value = !isMaximized.value
  }
}

const closeWindow = () => {
  appWindow.close()
}

const activeTab = ref('explorer') // 'explorer' | 'trash'
const showSidebar = ref(true)
const inspectorTab = ref('info') // 'info' | 'ai'

// Splitters dragging state
const isDraggingLeftSplitter = ref(false)
const isDraggingRightSplitter = ref(false)

// Address Bar editing state
const isEditingPath = ref(false)
const editablePath = ref('')
const pathInputRef = ref(null)
const renameInputRef = ref(null)

// Search query
const searchQuery = ref(galleryStore.filters.search || '')
watch(searchQuery, (newVal) => {
  galleryStore.setFilter('search', newVal)
})

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

const hasComfyWorkflow = ref(false)
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

watch(selectedFile, async (newFile) => {
  hasComfyWorkflow.value = false
  if (newFile) {
    renamingState.name = newFile.name
    if (isTauri && isImage(newFile)) {
      try {
        const metadata = await invoke('parse_ai_metadata', { path: newFile.path })
        if (metadata && metadata.workflow) {
          hasComfyWorkflow.value = true
        }
      } catch (e) {
        console.error('Failed to parse AI metadata for workflow check:', e)
      }
    } else if (!isTauri && newFile.name === 'comfyui_00124_.png') {
      // Mock workflow for browser verification
      hasComfyWorkflow.value = true
    }
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

// Fetch trash items
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
  activeTab.value = 'explorer'
  await navigationStore.navigateTo(path)
  galleryStore.setFiles(navigationStore.folders)
}

async function navigateToHome() {
  activeTab.value = 'explorer'
  const homeDrive = navigationStore.drives.find(d => d.path.startsWith('C:'))
  if (homeDrive) {
    await navigateTo(homeDrive.path)
  } else if (navigationStore.drives.length > 0) {
    await navigateTo(navigationStore.drives[0].path)
  }
}

async function goBack() {
  activeTab.value = 'explorer'
  await navigationStore.goBack()
  galleryStore.setFiles(navigationStore.folders)
}

async function goForward() {
  activeTab.value = 'explorer'
  await navigationStore.goForward()
  galleryStore.setFiles(navigationStore.folders)
}

async function goUp() {
  if (!navigationStore.currentPath) return
  const parts = navigationStore.currentPath.split('\\').filter(Boolean)
  if (parts.length > 1) {
    const parentPath = parts.slice(0, parts.length - 1).join('\\')
    await navigateTo(parentPath)
  } else {
    await navigateToHome()
  }
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

// Сортировка и фильтрация
function onSortChange() {
  // Реактивность Pinia автоматически обновит displayedFiles
}

function onFilterChange() {
  // Реактивность Pinia автоматически обновит displayedFiles
}

function toggleSortOrder() {
  galleryStore.sortOrder = galleryStore.sortOrder === 'asc' ? 'desc' : 'asc'
}

// Dialog functions
function openSettings() {
  router.push('/settings')
}

// Explorer File actions
async function createFolder() {
  const name = prompt('Введите имя папки:')
  if (name && navigationStore.currentPath) {
    const newPath = navigationStore.currentPath.endsWith('\\')
      ? navigationStore.currentPath + name
      : navigationStore.currentPath + '\\' + name
    try {
      await invoke('mkdir_folder', { path: newPath })
      await refreshData()
    } catch (e) {
      console.error(e)
      alert('Не удалось создать папку: ' + e)
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
  if (confirm(`Переместить файл "${file.name}" в корзину?`)) {
    try {
      await invoke('move_to_trash', { paths: [file.path] })
      await refreshData()
      galleryStore.clearSelection()
    } catch (err) {
      alert('Не удалось удалить файл: ' + err)
    }
  }
}

// Multiple deletes
async function deleteMultipleSelected() {
  if (confirm(`Переместить ${galleryStore.selectedIds.length} файлов в корзину?`)) {
    try {
      await invoke('move_to_trash', { paths: galleryStore.selectedIds })
      await refreshData()
      galleryStore.clearSelection()
    } catch (err) {
      alert('Не удалось удалить файлы: ' + err)
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
    alert('Не удалось восстановить файл: ' + err)
  }
}

async function deleteTrashFilePermanently(item) {
  if (confirm('Удалить файл навсегда? Это действие нельзя отменить.')) {
    try {
      await invoke('delete_file_system', { path: item.trashPath })
      const meta = item.trashPath.replace(/\.[^/.]+$/, "") + ".meta.json"
      await invoke('delete_file_system', { path: meta }).catch(() => {})
      await refreshData()
    } catch (err) {
      alert('Не удалось удалить файл: ' + err)
    }
  }
}

async function clearTrash() {
  if (confirm('Очистить корзину навсегда? Все файлы внутри будут удалены окончательно.')) {
    try {
      await invoke('empty_trash')
      await refreshData()
    } catch (err) {
      alert('Не удалось очистить корзину: ' + err)
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
  configStore.leftPanel.width = Math.max(160, Math.min(e.clientX - 6, window.innerWidth / 2))
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

// Address Bar click edit methods
function startEditingPath() {
  editablePath.value = navigationStore.currentPath || ''
  isEditingPath.value = true
  nextTick(() => {
    if (pathInputRef.value) {
      pathInputRef.value.focus()
      pathInputRef.value.select()
    }
  })
}

async function commitPathEdit() {
  setTimeout(async () => {
    isEditingPath.value = false
    if (editablePath.value && editablePath.value !== navigationStore.currentPath) {
      try {
        await navigateTo(editablePath.value)
      } catch (err) {
        console.error(err)
      }
    }
  }, 150)
}

// Focus Rename Input
function focusRenameInput() {
  if (!configStore.rightPanel.show) {
    configStore.rightPanel.show = true
  }
  nextTick(() => {
    const el = document.getElementById('rename-input-field')
    if (el) {
      el.focus()
      el.select()
    }
  })
}

// Theme toggling
function toggleTheme() {
  const nextAppearance = configStore.settings.appearance === 0 ? 1 : 0
  configStore.setAppearance(nextAppearance)
  setTheme(nextAppearance, nextAppearance === 0 ? configStore.settings.lightTheme : configStore.settings.darkTheme)
}

// Keyboard shortcuts global handler
function handleKeyDown(e) {
  // Игнорируем пробел/горячие клавиши, если фокус в инпуте
  const activeEl = document.activeElement
  const isInput = activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA' || activeEl.isContentEditable)
  if (isInput) return

  if (e.key === ' ' || e.code === 'Space') {
    e.preventDefault()
    const count = galleryStore.selectedIds.length
    if (count === 1) {
      if (selectedFile.value && (isImage(selectedFile.value) || isVideo(selectedFile.value))) {
        openQuickLook(selectedFile.value)
      }
    } else if (count >= 2 && count <= 6) {
      openCompare()
    }
  }

  if (e.key === 'k' && selectedFile.value && isImage(selectedFile.value)) {
    e.preventDefault()
    openCrop(selectedFile.value)
  }
  if (e.key === 'c' && galleryStore.selectedIds.length >= 2) {
    e.preventDefault()
    openCompare()
  }
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeyDown)
  await navigationStore.loadDrives()
  await fetchTrash()

  if (navigationStore.drives.length > 0) {
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