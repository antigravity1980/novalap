<template>
  <div class="file-icon shrink-0 flex items-center justify-center overflow-hidden rounded bg-base-200/50" :style="{ width: size + 'px', height: size + 'px' }">
    <img v-if="isFolder" :src="folderIconUrl" class="w-full h-full object-contain" />
    <img v-else-if="thumbnailUrl" :src="thumbnailUrl" class="w-full h-full object-cover" />
    <span v-else class="text-base-content/40 text-sm select-none">
      <span v-if="isVideo">📹</span>
      <span v-else>📄</span>
    </span>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from "vue";
import { useConfigStore } from "@/stores/configStore";
import { getCachedThumbnail } from "@/modules/gallery/explorerThumbnailsCache";
import { getPreviewUrl } from "@/common/utils";
import { invoke } from "@tauri-apps/api/core";

// Import all folder icons
import icon01 from "@/assets/folder-icons/01.svg?url";
import icon02 from "@/assets/folder-icons/02.svg?url";
import icon03 from "@/assets/folder-icons/03.svg?url";
import icon04 from "@/assets/folder-icons/04.svg?url";
import icon05 from "@/assets/folder-icons/05.svg?url";
import icon06 from "@/assets/folder-icons/06.svg?url";
import icon07 from "@/assets/folder-icons/07.svg?url";
import icon08 from "@/assets/folder-icons/08.svg?url";
import icon09 from "@/assets/folder-icons/09.svg?url";
import icon10 from "@/assets/folder-icons/10.svg?url";
import icon11 from "@/assets/folder-icons/11.svg?url";
import icon12 from "@/assets/folder-icons/12.svg?url";
import icon14 from "@/assets/folder-icons/14.svg?url";
import icon15 from "@/assets/folder-icons/15.svg?url";
import iconI1 from "@/assets/folder-icons/I1.svg?url";

const FOLDER_ICON_URLS = {
  "01.ico": icon01,
  "02.ico": icon02,
  "03.ico": icon03,
  "04.ico": icon04,
  "05.ico": icon05,
  "06.ico": icon06,
  "07.ico": icon07,
  "08.ico": icon08,
  "09.ico": icon09,
  "10.ico": icon10,
  "11.ico": icon11,
  "12.ico": icon12,
  "14.ico": icon14,
  "15.ico": icon15,
  "I1.ico": iconI1,
};

const props = defineProps({
  file: { type: Object, required: true },
  size: { type: Number, default: 24 }
});

const configStore = useConfigStore();

const isFolder = computed(() => {
  return props.file.is_dir === true || props.file.file_type === "directory" || props.file.is_directory === true;
});

const isImage = computed(() => {
  const ext = props.file.extension?.toLowerCase();
  return ["jpg", "jpeg", "png", "gif", "bmp", "webp", "avif", "jxl", "svg", "ico"].includes(ext);
});

const isVideo = computed(() => {
  const ext = props.file.extension?.toLowerCase();
  return ["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "mpeg", "3gp"].includes(ext);
});

const folderIconUrl = computed(() => {
  if (!isFolder.value) return "";
  const customIcon = configStore.folderIcons?.[props.file.path];
  return FOLDER_ICON_URLS[customIcon] || FOLDER_ICON_URLS["14.ico"];
});

const thumbnailUrl = ref("");

async function loadThumbnail() {
  if (isFolder.value) return;
  if (!isImage.value && !isVideo.value) return;

  const currentPath = props.file.path;
  const cacheKey = `${currentPath}__256`; // Use small bucket for lists/tables

  const cached = getCachedThumbnail(cacheKey);
  if (cached) {
    thumbnailUrl.value = cached;
    return;
  }

  // Fallback to fast loading for small images
  if (isImage.value && props.file.size < 200 * 1024) {
    const url = getPreviewUrl(0, currentPath);
    thumbnailUrl.value = url;
    return;
  }

  try {
    const url = await invoke("get_explorer_thumbnail", {
      path: currentPath,
      size: 256,
    });
    thumbnailUrl.value = url;
  } catch (e) {
    thumbnailUrl.value = "";
  }
}

onMounted(() => {
  loadThumbnail();
});

watch(() => props.file.path, () => {
  thumbnailUrl.value = "";
  loadThumbnail();
});
</script>
