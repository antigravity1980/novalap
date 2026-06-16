<template>
  <div class="trash-card p-3 rounded-xl bg-base-200 border border-neutral/20 flex gap-3.5 hover:bg-neutral/5 hover:border-neutral/30 transition-all duration-200 group relative">
    
    <!-- Left: Thumbnail/Icon Box -->
    <div class="w-20 h-20 rounded-lg overflow-hidden bg-base-300 border border-neutral/10 shrink-0 flex items-center justify-center relative select-none">
      <!-- Image preview -->
      <img
        v-if="isImage && thumbnailUrl"
        :src="thumbnailUrl"
        class="w-full h-full object-contain transition-transform duration-300 group-hover:scale-105"
        loading="lazy"
      />
      <!-- Video icon/visual representation -->
      <div v-else-if="isVideo" class="w-full h-full flex flex-col items-center justify-center gap-1 bg-slate-800/10 text-base-content/40">
        <span class="text-2xl filter drop-shadow">🎬</span>
        <span class="text-[8px] font-bold uppercase tracking-wider opacity-60">VIDEO</span>
      </div>
      <!-- Folder icon -->
      <div v-else-if="item.isDir" class="w-full h-full flex items-center justify-center bg-base-300/30">
        <img :src="icon14" class="w-10 h-10 object-contain pointer-events-none select-none" />
      </div>
      <!-- Other files -->
      <div v-else class="w-full h-full flex flex-col items-center justify-center bg-neutral/5 text-base-content/40">
        <span class="text-2xl">📄</span>
        <span class="text-[8px] uppercase font-bold tracking-wider mt-1">{{ extension }}</span>
      </div>
    </div>

    <!-- Right: Metadata & Actions -->
    <div class="flex-1 min-w-0 flex flex-col justify-between py-0.5">
      <div class="space-y-0.5">
        <h4 class="text-xs font-semibold text-base-content/90 truncate" :title="fileName">
          {{ fileName }}
        </h4>
        <p class="text-[10px] text-base-content/40 truncate leading-tight" :title="item.originalPath">
          {{ item.originalPath }}
        </p>
        <p class="text-[9px] text-base-content/30 leading-none">
          {{ $t('explorer.modified') }}: {{ formatDateString(item.deletedAt) }}
        </p>
      </div>

      <div class="flex items-center justify-between mt-2.5 pt-1.5 border-t border-neutral/10">
        <span class="text-[10px] font-mono text-base-content/50">{{ formatBytes(item.size) }}</span>
        
        <div class="flex gap-1.5">
          <!-- Split Restore Button Group -->
          <div class="join">
            <button
              class="btn btn-primary btn-xs px-2.5 h-6 min-h-0 join-item rounded-l text-[10px] font-semibold"
              @click="$emit('restore', item.trashPath)"
              :title="$t('explorer.restore')"
            >
              {{ $t('explorer.restore') }}
            </button>
            <button
              class="btn btn-primary btn-xs px-1.5 h-6 min-h-0 join-item rounded-r text-[10px] border-l border-primary-focus"
              @click="$emit('restore-to', item)"
              :title="$t('explorer.restore_to')"
            >
              ▼
            </button>
          </div>

          <!-- Permanent Delete Button -->
          <button
            class="btn btn-ghost btn-xs px-2 h-6 min-h-0 rounded text-[10px] text-error hover:bg-error/10"
            @click="$emit('delete-permanent', item)"
          >
            {{ $t('explorer.delete_permanently') }}
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, watch, onBeforeUnmount } from "vue";
import { getCachedThumbnail, setCachedThumbnail } from "@/modules/gallery/explorerThumbnailsCache";
import { getPreviewUrl } from "@/common/utils";
import { useUIStore } from "@/stores/uiStore";
import { invoke } from "@tauri-apps/api/core";
import icon14 from "@/assets/folder-icons/14.svg?url";

const props = defineProps({
  item: { type: Object, required: true },
});

defineEmits(["restore", "restore-to", "delete-permanent"]);

const uiStore = useUIStore();
const thumbnailUrl = ref("");
let debounceTimeout = null;

const fileName = computed(() => {
  const path = props.item.originalPath || props.item.trashPath;
  return path.split("\\").pop() || path.split("/").pop() || path;
});

const extension = computed(() => {
  const path = props.item.originalPath || props.item.trashPath;
  const idx = path.lastIndexOf(".");
  return idx !== -1 ? path.substring(idx + 1).toLowerCase() : "";
});

const isImage = computed(() => {
  return [
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "avif", "jxl", "svg", "ico"
  ].includes(extension.value);
});

const isVideo = computed(() => {
  return [
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "mpeg", "3gp"
  ].includes(extension.value);
});

async function loadThumbnail() {
  if (props.item.isDir || !isImage.value) {
    thumbnailUrl.value = "";
    return;
  }

  const currentPath = props.item.trashPath;
  const targetSize = 256;
  const cacheKey = `${currentPath}__${targetSize}`;

  const cached = getCachedThumbnail(cacheKey);
  if (cached) {
    thumbnailUrl.value = cached;
    return;
  }

  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }

  const ver = uiStore.thumbnailVersions[currentPath] || uiStore.fileVersions[currentPath] || 0;
  const versionQuery = ver ? `?v=${ver}` : "";

  if (props.item.size < 200 * 1024) {
    const url = getPreviewUrl(0, currentPath) + versionQuery;
    setCachedThumbnail(cacheKey, url);
    thumbnailUrl.value = url;
    return;
  }

  debounceTimeout = setTimeout(async () => {
    const cachedNow = getCachedThumbnail(cacheKey);
    if (cachedNow) {
      thumbnailUrl.value = cachedNow;
      return;
    }
    try {
      const url = await invoke("get_explorer_thumbnail", {
        path: currentPath,
        size: targetSize,
      });
      const busterUrl = url.startsWith("data:") ? url : url + versionQuery;
      setCachedThumbnail(cacheKey, busterUrl);
      if (props.item.trashPath === currentPath) {
        thumbnailUrl.value = busterUrl;
      }
    } catch (e) {
      if (props.item.trashPath === currentPath) {
        thumbnailUrl.value = "";
      }
    }
  }, 50);
}

onBeforeUnmount(() => {
  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }
});

watch(
  () => props.item.trashPath,
  loadThumbnail,
  { immediate: true },
);

function formatDateString(dateStr) {
  if (!dateStr) return "—";
  try {
    const d = new Date(dateStr);
    return d.toLocaleString();
  } catch {
    return dateStr;
  }
}

function formatBytes(bytes) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(0)} ${units[unitIndex]}`;
}
</script>

<style scoped>
.trash-card {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
