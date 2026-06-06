<template>
  <div class="filter-bar flex flex-wrap items-center gap-2 px-3 py-2 bg-base-200/50 rounded-lg">
    <!-- Поиск -->
    <div class="search-box flex-1 min-w-[200px]">
      <input
        type="text"
        v-model="searchQuery"
        :placeholder="$t('gallery.search_files_placeholder')"
        class="input input-bordered input-sm w-full"
        @input="onSearchChange"
      />
    </div>

    <!-- Сортировка -->
    <select v-model="galleryStore.sortBy" class="select select-bordered select-sm" @change="onSortChange">
      <option value="name">{{ $t('gallery.sort_options.name') }}</option>
      <option value="size">{{ $t('gallery.sort_options.size') }}</option>
      <option value="date">{{ $t('gallery.sort_options.date') }}</option>
      <option value="resolution">{{ $t('gallery.sort_options.resolution') }}</option>
      <option value="ai_source">{{ $t('gallery.sort_options.ai_source') }}</option>
    </select>

    <button class="btn btn-ghost btn-sm" @click="toggleSortOrder">
      {{ galleryStore.sortOrder === 'asc' ? '↑' : '↓' }}
    </button>

    <div class="divider divider-horizontal mx-0"></div>

    <!-- Фильтр по формату -->
    <select v-model="galleryStore.filters.format" class="select select-bordered select-sm" @change="onFilterChange">
      <option value="">{{ $t('explorer.all_formats') }}</option>
      <option value="png">PNG</option>
      <option value="jpg">JPEG</option>
      <option value="jpeg">JPEG</option>
      <option value="webp">WebP</option>
      <option value="gif">GIF</option>
      <option value="mp4">MP4</option>
      <option value="mkv">MKV</option>
      <option value="webm">WebM</option>
      <option value="mov">MOV</option>
    </select>

    <!-- Фильтр по AI-источнику -->
    <select v-model="galleryStore.filters.aiSource" class="select select-bordered select-sm" @change="onFilterChange">
      <option value="">{{ $t('explorer.all_sources') }}</option>
      <option value="ComfyUI">ComfyUI</option>
      <option value="Midjourney">Midjourney</option>
      <option value="Stable Diffusion">Stable Diffusion</option>
      <option value="Nano Banana">Nano Banana</option>
      <option value="GPT Images">GPT Images</option>
      <option value="Grok Image">Grok Image</option>
      <option value="DALL-E">DALL-E</option>
      <option value="Krita AI">Krita AI</option>
      <option value="Unknown">{{ $t('explorer.unknown_source') }}</option>
    </select>

    <!-- Кол-во выбранных -->
    <span class="text-xs text-base-content/50 ml-auto whitespace-nowrap">
      {{ $t('gallery.selected_count', { count: galleryStore.selectedIds.length }) }}
    </span>

    <button class="btn btn-ghost btn-xs" @click="galleryStore.clearSelection()">✕</button>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useGalleryStore } from '../store'

const galleryStore = useGalleryStore()
const searchQuery = ref(galleryStore.filters.search)

function onSearchChange() {
  galleryStore.setFilter('search', searchQuery.value)
}

function onSortChange() {
  galleryStore.setSorting(galleryStore.sortBy, galleryStore.sortOrder)
}

function toggleSortOrder() {
  const newOrder = galleryStore.sortOrder === 'asc' ? 'desc' : 'asc'
  galleryStore.setSorting(galleryStore.sortBy, newOrder)
}

function onFilterChange() {
  // Фильтры применяются через геттер displayedFiles
}
</script>

<style scoped>
.filter-bar {
  backdrop-filter: blur(8px);
}
</style>