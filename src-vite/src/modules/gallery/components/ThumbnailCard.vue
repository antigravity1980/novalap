<template>
  <div
    class="thumbnail-card rounded-xl overflow-hidden border cursor-pointer transition-all duration-200 bg-base-200/40 relative flex flex-col justify-between"
    :class="{
      'border-primary ring-2 ring-primary/45 shadow-lg shadow-primary/10 translate-y-[-2px] bg-base-100':
        selected && !dragOverCard,
      'border-base-content/5 hover:border-primary/20 hover:shadow-xl hover:translate-y-[-2px] hover:bg-base-100/30':
        !selected && !dragOverCard,
      'bg-secondary/20 border-secondary ring-2 ring-secondary/45 border-dashed':
        dragOverCard,
      'opacity-50 grayscale-[35%]': isCut,
    }"
    :style="{ width: size + 'px' }"
    @click="$emit('click', $event)"
    @dblclick="$emit('dblclick', $event)"
    @contextmenu.prevent.stop="handleContextMenu($event)"
    @mouseenter="onHoverEnrich"
    @focusin="onHoverEnrich"
    draggable="true"
    @dragstart="handleDragStart"
    @dragover.prevent="isFolder ? (dragOverCard = true) : null"
    @dragenter.prevent="isFolder ? (dragOverCard = true) : null"
    @dragleave="dragOverCard = false"
    @drop.prevent="handleDrop"
    tabindex="0"
  >
    <!-- Thumbnail Image Container -->
    <div
      class="thumbnail-image bg-base-300 flex items-center justify-center overflow-hidden relative select-none w-full"
      :style="{ height: size * 0.75 + 'px' }"
    >
      <img
        v-if="isImage && thumbnailUrl"
        :src="thumbnailUrl"
        :alt="file.name"
        class="w-full h-full object-contain transition-transform duration-300 hover:scale-105"
        loading="lazy"
      />
      <!-- Video tag/icon overlay -->
      <div
        v-else-if="isVideo"
        class="w-full h-full flex flex-col items-center justify-center gap-1.5 bg-gradient-to-br from-base-300 to-base-200 text-base-content/40 hover:text-base-content/60"
      >
        <span class="text-3xl filter drop-shadow">🎬</span>
        <span
          class="text-[10px] uppercase font-bold tracking-wider opacity-60"
          >{{ $t("gallery.video_label") }}</span
        >
        <div
          class="absolute bottom-2 right-2 bg-black/60 backdrop-blur rounded px-1.5 py-0.5 text-[10px] text-white font-mono flex items-center gap-1"
        >
          <span>▶</span>
          <span>{{ $t("gallery.video_label").toUpperCase() }}</span>
        </div>
      </div>
      <!-- Папка -->
      <div
        v-else-if="isFolder"
        class="w-full h-full flex flex-col items-center justify-center bg-base-300/30"
      >
        <img
          :src="folderIconUrl"
          class="w-16 h-16 object-contain select-none pointer-events-none"
        />
      </div>
      <!-- Other files generic -->
      <div
        v-else
        class="w-full h-full flex flex-col items-center justify-center gap-1.5 bg-base-300/30 text-base-content/40"
      >
        <span class="text-3xl">📄</span>
        <span class="text-[10px] uppercase font-bold tracking-wider">{{
          file.extension?.toUpperCase() ||
          $t("gallery.file_label").toUpperCase()
        }}</span>
      </div>

      <!-- AI Source indicator (corner badge) -->
      <div
        v-if="aiSourceBadge"
        class="absolute top-2 left-2 z-10 flex items-center gap-1"
      >
        <span
          class="badge badge-xs text-[9px] font-bold py-1 px-1.5 border border-white/10 shadow shadow-black/20"
          :class="getAiSourceClass(aiSourceBadge)"
        >
          {{ aiSourceBadge }}
        </span>
        <span
          v-if="isAiSourcePending"
          class="inline-block w-2.5 h-2.5 rounded-full border border-white/40 border-t-transparent animate-spin"
          :title="$t('gallery.loading_ai_source') || 'определяю AI…'"
        ></span>
      </div>

      <!-- ComfyUI Badge in Top-Right Corner -->
      <div
        v-if="file.ai_source === 'ComfyUI'"
        class="absolute top-2 z-10 transition-all duration-200"
        :class="selected ? 'right-8' : 'right-2'"
      >
        <span
          class="bg-yellow-400 text-black text-[9.5px] font-extrabold py-0.5 px-1.5 rounded shadow border border-yellow-500"
        >
          ComfyUI
        </span>
      </div>

      <!-- Selection checkmark badge -->
      <div
        v-if="selected"
        class="absolute top-2 right-2 bg-primary text-primary-content w-4 h-4 rounded-full flex items-center justify-center text-[9px] font-bold border border-white/20 shadow-md z-20"
      >
        ✓
      </div>
    </div>

    <!-- Info row -->
    <div
      class="thumbnail-info p-2.5 text-xs flex flex-col gap-1 shrink-0 border-t border-base-content/5 bg-base-200/20"
    >
      <div v-if="isRenaming" class="w-full">
        <input
          ref="renameInputRef"
          v-model="renameText"
          type="text"
          class="input input-xs input-bordered w-full text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary bg-base-100 text-base-content px-1.5 py-0.5 rounded"
          @keydown.enter.stop="saveRename"
          @keydown.esc.stop="cancelRename"
          @blur="saveRename"
          @click.stop
        />
      </div>
      <div
        v-else
        class="file-name truncate font-medium text-base-content/90"
        :title="file.name"
      >
        {{ file.name }}
      </div>
      <div
        class="flex items-center justify-between mt-0.5 text-[10px] text-base-content/40 font-mono"
      >
        <span
          v-if="isFolder"
          class="font-semibold truncate pr-1 flex items-center gap-1"
        >
          <span>{{ folderCountsText }}</span>
          <span
            v-if="isCountPending"
            class="inline-block w-2.5 h-2.5 rounded-full border border-base-content/30 border-t-transparent animate-spin shrink-0"
            :title="$t('gallery.loading_counts') || 'считаю...'"
          ></span>
        </span>
        <span v-else-if="file.resolution" class="font-semibold">
          {{ file.resolution.width }}×{{ file.resolution.height }}
        </span>
        <span v-else>—</span>
        <span>{{ isFolder ? "" : formatBytes(file.size) }}</span>
      </div>
    </div>

    <MessageBox
      v-if="showConfirm"
      :title="$t('settings.general.delete_confirm_title') || 'Удаление'"
      :message="deleteMessage"
      :OkText="$t('explorer.delete') || 'Удалить'"
      :cancelText="$t('batch_ops.cancel') || 'Отмена'"
      :warningOk="true"
      checkboxText="Больше не спрашивать"
      :checkboxChecked="skipDeleteCheckboxVal"
      @checkbox-change="(val) => (skipDeleteCheckboxVal = val)"
      @ok="confirmDeletion"
      @cancel="showConfirm = false"
    />

    <ContextMenu
      ref="contextMenuRef"
      :menuItems="contextMenuItems"
      :smallIcon="true"
      style="display: none"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onBeforeUnmount } from "vue";
import { getAssetSrc, getPreviewUrl } from "@/common/utils";
import {
  getCachedThumbnail,
  setCachedThumbnail,
  clearCachedThumbnailByKey,
} from "@/modules/gallery/explorerThumbnailsCache";
import { useConfigStore } from "@/stores/configStore";
import { useNavigationStore } from "@/modules/navigation/store";
import { useGalleryStore } from "@/modules/gallery/store";
import { useUIStore } from "@/stores/uiStore";
import { invoke } from "@tauri-apps/api/core";
import ContextMenu from "@/components/ContextMenu.vue";
import MessageBox from "@/components/MessageBox.vue";
import icon01 from "@/assets/folder-icons/01.ico";
import icon02 from "@/assets/folder-icons/02.ico";
import icon03 from "@/assets/folder-icons/03.ico";
import icon04 from "@/assets/folder-icons/04.ico";
import icon05 from "@/assets/folder-icons/05.ico";
import icon06 from "@/assets/folder-icons/06.ico";
import icon07 from "@/assets/folder-icons/07.ico";
import icon08 from "@/assets/folder-icons/08.ico";
import icon09 from "@/assets/folder-icons/09.ico";
import icon10 from "@/assets/folder-icons/10.ico";
import icon11 from "@/assets/folder-icons/11.ico";
import icon12 from "@/assets/folder-icons/12.ico";
import icon14 from "@/assets/folder-icons/14.ico";
import icon15 from "@/assets/folder-icons/15.ico";
import iconI1 from "@/assets/folder-icons/I1.ico";

const props = defineProps({
  file: { type: Object, required: true },
  size: { type: Number, default: 200 },
  selected: { type: Boolean, default: false },
});

function onHoverEnrich() {
  if (galleryStore.needsEnrichment(props.file)) {
    galleryStore.requestEnrichments([props.file.path]);
  }
}

watch(
  () => props.selected,
  (isSelected) => {
    if (isSelected) onHoverEnrich();
  },
);

const isCountPending = computed(() => {
  if (!isFolder.value) return false;
  return props.file.dir_count == null || props.file.file_count == null;
});

const aiSourceBadge = computed(() => {
  if (isFolder.value) return null;
  return props.file.ai_source && props.file.ai_source !== "ComfyUI"
    ? props.file.ai_source
    : null;
});

const isAiSourcePending = computed(() => {
  if (isFolder.value) return false;
  if (props.file.ai_source) return false;
  const ext = (props.file.extension || "").toLowerCase();
  if (!ext) return false;
  return /\.(png|jpe?g|webp|tiff?|avif|heic|heif|jxl|gif)$/.test(ext);
});

defineEmits(["click", "dblclick"]);

const configStore = useConfigStore();
const navigationStore = useNavigationStore();
const galleryStore = useGalleryStore();
const uiStore = useUIStore();
const contextMenuRef = ref(null);

const isRenaming = ref(false);
const renameText = ref("");
const dragOverCard = ref(false);
const renameInputRef = ref(null);

const isCut = computed(() => {
  return (
    galleryStore.clipboard.action === "cut" &&
    galleryStore.clipboard.paths.includes(props.file.path)
  );
});

const isFolder = computed(() => {
  return (
    props.file.is_dir === true ||
    props.file.file_type === "directory" ||
    props.file.is_directory === true
  );
});

const isImage = computed(() => {
  const ext = props.file.extension?.toLowerCase();
  return [
    "jpg",
    "jpeg",
    "png",
    "gif",
    "bmp",
    "webp",
    "avif",
    "jxl",
    "svg",
    "ico",
  ].includes(ext);
});

const isVideo = computed(() => {
  const ext = props.file.extension?.toLowerCase();
  return [
    "mp4",
    "mkv",
    "avi",
    "mov",
    "webm",
    "flv",
    "wmv",
    "mpeg",
    "3gp",
  ].includes(ext);
});

// Async loading of generated thumbnails for large files to prevent UI freeze
const thumbnailUrl = ref("");
let debounceTimeout = null;
const sizeBuckets = [256, 512, 1024];

async function loadThumbnail() {
  if (!isImage.value) return;

  const currentPath = props.file.path;
  const targetSize = sizeBuckets.find(b => b >= props.size) || 1024;
  const cacheKey = `${currentPath}__${targetSize}`;

  const cached = getCachedThumbnail(cacheKey);
  if (cached) {
    thumbnailUrl.value = cached;
    return;
  }

  for (const bucket of sizeBuckets) {
    if (bucket >= props.size) {
      const altKey = `${currentPath}__${bucket}`;
      const altCached = getCachedThumbnail(altKey);
      if (altCached) {
        thumbnailUrl.value = altCached;
        return;
      }
    }
  }

  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }

  if (props.file.size < 200 * 1024) {
    const url = getPreviewUrl(0, currentPath);
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
      setCachedThumbnail(cacheKey, url);
      if (props.file.path === currentPath) {
        thumbnailUrl.value = url;
      }
    } catch (e) {
      if (props.file.path === currentPath) {
        thumbnailUrl.value = "";
      }
    }
  }, 100);
}

onBeforeUnmount(() => {
  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }
});
watch(
  [
    () => props.file.path,
    () => props.size,
    () => uiStore.fileVersions[props.file.path],
    () => uiStore.thumbnailVersions[props.file.path],
  ],
  loadThumbnail,
  { immediate: true },
);

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

const FOLDER_ICON_MENU_ITEMS = [
  {
    iconName: null,
    iconUrl: FOLDER_ICON_URLS["14.ico"],
    tooltip: "По умолчанию",
  },
  {
    iconName: "I1.ico",
    iconUrl: FOLDER_ICON_URLS["I1.ico"],
    tooltip: "Важная (Звезда)",
  },
  {
    iconName: "01.ico",
    iconUrl: FOLDER_ICON_URLS["01.ico"],
    tooltip: "Папка 01",
  },
  {
    iconName: "02.ico",
    iconUrl: FOLDER_ICON_URLS["02.ico"],
    tooltip: "Папка 02",
  },
  {
    iconName: "03.ico",
    iconUrl: FOLDER_ICON_URLS["03.ico"],
    tooltip: "Папка 03",
  },
  {
    iconName: "04.ico",
    iconUrl: FOLDER_ICON_URLS["04.ico"],
    tooltip: "Папка 04",
  },
  {
    iconName: "05.ico",
    iconUrl: FOLDER_ICON_URLS["05.ico"],
    tooltip: "Папка 05",
  },
  {
    iconName: "06.ico",
    iconUrl: FOLDER_ICON_URLS["06.ico"],
    tooltip: "Папка 06",
  },
  {
    iconName: "07.ico",
    iconUrl: FOLDER_ICON_URLS["07.ico"],
    tooltip: "Папка 07",
  },
  {
    iconName: "08.ico",
    iconUrl: FOLDER_ICON_URLS["08.ico"],
    tooltip: "Папка 08",
  },
  {
    iconName: "09.ico",
    iconUrl: FOLDER_ICON_URLS["09.ico"],
    tooltip: "Папка 09",
  },
  {
    iconName: "10.ico",
    iconUrl: FOLDER_ICON_URLS["10.ico"],
    tooltip: "Папка 10",
  },
  {
    iconName: "11.ico",
    iconUrl: FOLDER_ICON_URLS["11.ico"],
    tooltip: "Папка 11",
  },
  {
    iconName: "12.ico",
    iconUrl: FOLDER_ICON_URLS["12.ico"],
    tooltip: "Папка 12",
  },
  {
    iconName: "15.ico",
    iconUrl: FOLDER_ICON_URLS["15.ico"],
    tooltip: "Папка 15",
  },
];

const folderIconUrl = computed(() => {
  if (!isFolder.value) return "";
  const customIcon = configStore.folderIcons?.[props.file.path];
  return FOLDER_ICON_URLS[customIcon] || FOLDER_ICON_URLS["14.ico"];
});

// Pluralization for Russian/English subfolders and files count
const folderCountsText = computed(() => {
  if (!isFolder.value) return "";
  const dirs = props.file.dir_count || 0;
  const files = props.file.file_count || 0;

  const getRussianPlural = (num, one, two, five) => {
    let n = Math.abs(num);
    n %= 100;
    if (n >= 5 && n <= 20) return five;
    n %= 10;
    if (n === 1) return one;
    if (n >= 2 && n <= 4) return two;
    return five;
  };

  const foldersStr =
    configStore.settings.language === "ru"
      ? `${dirs} ${getRussianPlural(dirs, "папка", "папки", "папок")}`
      : `${dirs} folder${dirs !== 1 ? "s" : ""}`;

  const filesStr =
    configStore.settings.language === "ru"
      ? `${files} ${getRussianPlural(files, "файл", "файла", "файлов")}`
      : `${files} file${files !== 1 ? "s" : ""}`;

  return `${foldersStr}, ${filesStr}`;
});

const contextMenuItems = computed(() => {
  const items = [
    { label: "Переименовать", action: () => renameItem() },
    { label: "Копировать", action: () => copyItem() },
    { label: "Вырезать", action: () => cutItem() },
    { label: "Удалить", action: () => deleteItem() },
  ];

  if (galleryStore.clipboard.paths.length > 0 && isFolder.value) {
    items.push({
      label: "Вставить",
      action: () => galleryStore.paste(props.file.path),
    });
  }

  // Compare stack (cross-folder)
  if (!isFolder.value) {
    const inStack = galleryStore.isInCompareStack(props.file.path);
    if (inStack) {
      items.push({ separator: true });
      items.push({
        label: "Убрать из сравнения",
        action: () => galleryStore.removeFromCompare(props.file.path),
      });
    } else if (galleryStore.canAddMoreToCompare()) {
      items.push({ separator: true });
      items.push({
        label: "Добавить к сравнению",
        action: () => galleryStore.addToCompare(props.file),
      });
    }
  }

  if (isFolder.value) {
    const isFav = configStore.settings.favorites?.includes(props.file.path);
    items.push({
      label: isFav ? "Удалить из избранного" : "Добавить в избранное",
      action: () => configStore.toggleFavorite(props.file.path),
    });
    items.push({ separator: true });
    items.push({
      label: "Перекрасить папку",
      grid: true,
      children: FOLDER_ICON_MENU_ITEMS.map(
        ({ iconName, iconUrl, tooltip }) => ({
          iconUrl,
          tooltip,
          action: () => setFolderIcon(iconName),
        }),
      ),
    });
  }
  return items;
});

watch(
  () => galleryStore.renamingPath,
  (newVal) => {
    if (newVal === props.file.path) {
      startRename();
    } else {
      isRenaming.value = false;
    }
  },
  { immediate: true },
);

function startRename() {
  isRenaming.value = true;
  renameText.value = props.file.name;
  nextTick(() => {
    if (renameInputRef.value) {
      renameInputRef.value.focus();
      const dotIndex = props.file.name.lastIndexOf(".");
      if (dotIndex > 0 && !isFolder.value) {
        renameInputRef.value.setSelectionRange(0, dotIndex);
      } else {
        renameInputRef.value.select();
      }
    }
  });
}

async function saveRename() {
  if (!isRenaming.value) return;
  isRenaming.value = false;
  galleryStore.renamingPath = null;
  const newName = renameText.value.trim();
  if (!newName || newName === props.file.name) return;

  const lastSlash = Math.max(
    props.file.path.lastIndexOf("\\"),
    props.file.path.lastIndexOf("/"),
  );
  const parentPath =
    lastSlash !== -1 ? props.file.path.substring(0, lastSlash) : "";
  const separator = props.file.path.includes("/") ? "/" : "\\";
  const newPath = parentPath ? `${parentPath}${separator}${newName}` : newName;
  try {
    await invoke("cross_move", { src: props.file.path, dest: newPath });
    await navigationStore.navigateTo(navigationStore.currentPath);
    galleryStore.setFiles(navigationStore.folders);
  } catch (e) {
    alert("Ошибка переименования: " + e);
  }
}

function cancelRename() {
  isRenaming.value = false;
  galleryStore.renamingPath = null;
}

function renameItem() {
  galleryStore.renamingPath = props.file.path;
}

function handleDragStart(e) {
  let paths = [];
  if (props.selected) {
    paths = [...galleryStore.selectedIds];
  } else {
    paths = [props.file.path];
  }
  e.dataTransfer.setData("text/plain", JSON.stringify(paths));
  e.dataTransfer.effectAllowed = "move";
}

async function handleDrop(e) {
  dragOverCard.value = false;
  if (!isFolder.value) return;

  try {
    const data = e.dataTransfer.getData("text/plain");
    if (!data) return;
    const paths = JSON.parse(data);
    if (!Array.isArray(paths) || paths.length === 0) return;

    const destPath = props.file.path;
    for (const src of paths) {
      if (src === destPath) continue;
      const lastSlash = Math.max(src.lastIndexOf("\\"), src.lastIndexOf("/"));
      const fileName = lastSlash !== -1 ? src.substring(lastSlash + 1) : src;
      const dest = `${destPath}${destPath.endsWith("\\") || destPath.endsWith("/") ? "" : "\\"}${fileName}`;
      if (src.toLowerCase() === dest.toLowerCase()) continue;

      await invoke("cross_move", { src, dest });
    }

    await navigationStore.navigateTo(navigationStore.currentPath);
    galleryStore.setFiles(navigationStore.folders);
  } catch (err) {
    console.error("Card drop failed:", err);
  }
}

const showConfirm = ref(false);
const skipDeleteCheckboxVal = ref(false);

const deleteMessage = computed(() => {
  const count = props.selected ? galleryStore.selectedIds.length : 1;
  if (count > 1) {
    return `Вы действительно хотите удалить эти ${count} элементов?`;
  }
  return `Вы действительно хотите удалить "${props.file.name}"?`;
});

async function performDelete() {
  const paths = props.selected
    ? [...galleryStore.selectedIds]
    : [props.file.path];
  try {
    await galleryStore.deleteFiles(paths);
    await navigationStore.navigateTo(navigationStore.currentPath);
    galleryStore.setFiles(navigationStore.folders);
  } catch (e) {
    alert("Ошибка удаления: " + e);
  }
}

function confirmDeletion() {
  showConfirm.value = false;
  if (skipDeleteCheckboxVal.value) {
    configStore.settings.skipDeleteConfirmation = true;
  }
  performDelete();
}

async function deleteItem() {
  if (configStore.settings.skipDeleteConfirmation) {
    await performDelete();
  } else {
    skipDeleteCheckboxVal.value = false;
    showConfirm.value = true;
  }
}

function copyItem() {
  const paths = props.selected
    ? [...galleryStore.selectedIds]
    : [props.file.path];
  galleryStore.setClipboard("copy", paths);
}

function cutItem() {
  const paths = props.selected
    ? [...galleryStore.selectedIds]
    : [props.file.path];
  galleryStore.setClipboard("cut", paths);
}

function handleContextMenu(e) {
  contextMenuRef.value?.open(e.clientX, e.clientY);
}

function setFolderIcon(iconName) {
  configStore.setFolderIcon(props.file.path, iconName);
}

function formatBytes(bytes) {
  if (!bytes) return "";
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(0)} ${units[unitIndex]}`;
}

function getAiSourceClass(source) {
  const s = source.toLowerCase();
  if (s.includes("comfyui")) {
    return "bg-yellow-500 text-black font-bold";
  }
  if (s.includes("midjourney")) {
    return "bg-blue-600/90 text-white";
  }
  if (s.includes("stable diffusion") || s.includes("sd_")) {
    return "bg-teal-600/90 text-white";
  }
  if (s.includes("gpt")) {
    return "bg-emerald-600/90 text-white";
  }
  if (s.includes("grok")) {
    return "bg-amber-600/90 text-white";
  }
  if (s.includes("nano banana")) {
    return "bg-rose-600/90 text-white";
  }
  return "bg-neutral-600/90 text-white";
}
</script>

<style scoped>
.thumbnail-card {
  transition:
    transform 0.2s cubic-bezier(0.4, 0, 0.2, 1),
    box-shadow 0.2s,
    border-color 0.2s,
    background-color 0.2s,
    opacity 0.2s,
    filter 0.2s;
}
.thumbnail-image img {
  pointer-events: none;
  backface-visibility: hidden;
}
</style>
