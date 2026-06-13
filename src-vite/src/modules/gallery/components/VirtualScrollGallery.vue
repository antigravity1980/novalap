<template>
  <div
    ref="containerRef"
    class="virtual-scroll-gallery h-full overflow-y-auto overflow-x-hidden focus:outline-none px-2 py-4"
    tabindex="0"
    @scroll="onScroll"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseLeave"
    @keydown="onKeyDown"
    @contextmenu.prevent.stop="handleContextMenu"
    style="user-select: none"
  >
    <div :style="{ height: totalHeight + 'px', position: 'relative' }">
      <div
        v-for="item in visibleItems"
        :key="item.key"
        :style="{
          position: 'absolute',
          top: item.top + 'px',
          left: '0px',
          right: '0px',
        }"
      >
        <div
          v-if="item.type === 'header'"
          class="group-header flex items-center gap-2 px-4 h-10 text-xs font-bold border-b border-neutral/20 text-base-content/60 select-none hover:bg-neutral/5 hover:text-base-content cursor-pointer transition-colors duration-150 rounded"
          @click="onGroupHeaderClick($event, item)"
        >
          <svg
            class="w-3 h-3 shrink-0 transition-transform duration-200"
            :class="collapsedGroups[item.title] ? '-rotate-90' : ''"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 4.5L6 7.5L9 4.5" />
          </svg>
          <span>{{ item.title }} ({{ item.fileCount }})</span>
        </div>

        <div
          v-else
          class="grid"
          :style="{
            gridTemplateColumns: `repeat(${colsPerRow}, minmax(0, ${thumbnailSize}px))`,
            gap: galleryStore.thumbnailGap + 'px',
            justifyContent: 'start',
          }"
        >
          <ThumbnailCard
            v-for="file in item.files"
            :key="file.path"
            :file="file"
            :size="thumbnailSize"
            :selected="galleryStore.selectedIds.includes(file.path)"
            @click.stop="onCardClick($event, file)"
            @dblclick.stop="onCardDblClick(file)"
            class="shrink-0"
            :style="{ width: thumbnailSize + 'px' }"
            :data-path="file.path"
          />
        </div>
      </div>
    </div>

    <div v-if="isDragging" class="rubber-band" :style="rubberBandStyle" />

    <ContextMenu
      ref="contextMenuRef"
      :menuItems="contextMenuItems"
      :smallIcon="true"
      style="display: none"
    />
  </div>
</template>

<script setup>
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  reactive,
  watch,
  nextTick,
} from "vue";
import { useGalleryStore, groupFilesHelper } from "../store";
import { useNavigationStore } from "../../navigation/store";
import { invoke } from "@tauri-apps/api/core";
import ThumbnailCard from "./ThumbnailCard.vue";
import ContextMenu from "@/components/ContextMenu.vue";

const props = defineProps({
  files: { type: Array, default: () => [] },
  thumbnailSize: { type: Number, default: 200 },
});

const emit = defineEmits(["openQuickLook"]);

const galleryStore = useGalleryStore();
const navigationStore = useNavigationStore();
const containerRef = ref(null);
const contextMenuRef = ref(null);
const containerWidth = ref(1200);
const containerHeight = ref(800);
const scrollTop = ref(0);
const gap = computed(() => galleryStore.thumbnailGap ?? 11);
const collapsedGroups = reactive({});
const BUFFER_ROWS = 5;

function toggleGroupCollapse(title) {
  collapsedGroups[title] = !collapsedGroups[title];
}

const colsPerRow = computed(() => {
  const width = Math.max(320, containerWidth.value || 1200);
  return Math.max(
    1,
    Math.floor((width - 16 + gap.value) / (props.thumbnailSize + gap.value)),
  );
});

const HEADER_HEIGHT = 40;
const ROW_HEIGHT_OFFSET = 42;

const layoutRows = computed(() => {
  const rows = [];
  const cols = colsPerRow.value;
  const rowH = Math.round(props.thumbnailSize * 0.75) + ROW_HEIGHT_OFFSET;

  if (!galleryStore.groupBy || galleryStore.groupBy === "none") {
    const files = props.files;
    for (let i = 0; i < files.length; i += cols) {
      rows.push({
        type: "file-row",
        files: files.slice(i, i + cols),
        height: rowH,
      });
    }
    return rows;
  }

  const groups = groupFilesHelper(props.files, galleryStore.groupBy);
  for (const group of groups) {
    if (!group.files.length) continue;
    rows.push({
      type: "header",
      title: group.title,
      fileCount: group.files.length,
      filePaths: group.files.map((f) => f.path),
      height: HEADER_HEIGHT,
    });
    if (!collapsedGroups[group.title]) {
      const files = group.files;
      for (let i = 0; i < files.length; i += cols) {
        rows.push({
          type: "file-row",
          files: files.slice(i, i + cols),
          height: rowH,
        });
      }
    }
  }
  return rows;
});

const totalHeight = computed(() => {
  let h = 0;
  for (const row of layoutRows.value) {
    h += row.height + gap.value;
  }
  return h;
});

const visibleItems = computed(() => {
  const rows = layoutRows.value;
  const g = gap.value;
  const viewTop = scrollTop.value;
  const viewBottom = viewTop + containerHeight.value;

  let y = 0;
  const result = [];
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    const rowBottom = y + row.height;

    if (rowBottom >= viewTop - BUFFER_ROWS * (props.thumbnailSize + g) &&
        y <= viewBottom + BUFFER_ROWS * (props.thumbnailSize + g)) {
      if (row.type === "header") {
        result.push({
          key: `header:${row.title}`,
          type: "header",
          title: row.title,
          fileCount: row.fileCount,
          filePaths: row.filePaths,
          top: y,
        });
      } else {
        result.push({
          key: `row:${i}:${row.files[0]?.path}`,
          type: "file-row",
          files: row.files,
          top: y,
        });
      }
    }
    y += row.height + g;
  }
  return result;
});

function onScroll(e) {
  scrollTop.value = e.target.scrollTop;
}

async function loadEnrichments() {
  const visibleFiles = props.files.filter((file) =>
    galleryStore.needsEnrichment(file),
  );
  if (visibleFiles.length === 0) return;
  await galleryStore.requestEnrichments(
    visibleFiles.slice(0, 60).map((f) => f.path),
  );
}

let prevFileCount = 0;
let prevFileKey = "";
watch(() => props.files, (newFiles) => {
  const newKey = newFiles.length + ":" + (newFiles[0]?.path || "");
  if (newKey === prevFileKey && newFiles.length === prevFileCount) return;
  prevFileKey = newKey;
  prevFileCount = newFiles.length;
  loadEnrichments();
}, { immediate: true });

let primeDebounce = null;
let prevSelectionKey = "";
watch(
  () => galleryStore.selectedIds.length,
  () => {
    const newKey = galleryStore.selectedIds.join("|");
    if (newKey === prevSelectionKey) return;
    prevSelectionKey = newKey;
    if (primeDebounce) clearTimeout(primeDebounce);
    primeDebounce = setTimeout(
      () => galleryStore.primeSelectedForEnrichment(),
      200,
    );
  },
);

const isDragging = ref(false);
const dragStart = reactive({ x: 0, y: 0 });
const dragCurrent = reactive({ x: 0, y: 0 });
let selectionBeforeDrag = [];
let cardRects = [];

const rubberBandStyle = computed(() => {
  const left = Math.min(dragStart.x, dragCurrent.x);
  const top = Math.min(dragStart.y, dragCurrent.y);
  const width = Math.abs(dragCurrent.x - dragStart.x);
  const height = Math.abs(dragCurrent.y - dragStart.y);
  return {
    left: left + "px",
    top: top + "px",
    width: width + "px",
    height: height + "px",
  };
});

function cacheCardRects() {
  const container = containerRef.value;
  if (!container) return;
  const containerRect = container.getBoundingClientRect();
  cardRects = Array.from(
    container.querySelectorAll(".thumbnail-card[data-path]"),
  ).map((el) => {
    const rect = el.getBoundingClientRect();
    return {
      path: el.dataset.path,
      left: rect.left - containerRect.left,
      top: rect.top - containerRect.top + container.scrollTop,
      right: rect.right - containerRect.left,
      bottom: rect.bottom - containerRect.top + container.scrollTop,
    };
  });
}

function getFilesInRect(rx1, ry1, rx2, ry2) {
  const left = Math.min(rx1, rx2);
  const right = Math.max(rx1, rx2);
  const top = Math.min(ry1, ry2);
  const bottom = Math.max(ry1, ry2);

  return cardRects
    .filter(
      (r) =>
        r.right > left && r.left < right && r.bottom > top && r.top < bottom,
    )
    .map((r) => r.path);
}

function onMouseDown(e) {
  if (e.button !== 0) return;
  const target = e.target;
  if (target.closest(".thumbnail-card") || target.closest(".group-header"))
    return;

  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top + containerRef.value.scrollTop;

  cacheCardRects();

  isDragging.value = true;
  dragStart.x = x;
  dragStart.y = y;
  dragCurrent.x = x;
  dragCurrent.y = y;

  if (!e.ctrlKey && !e.shiftKey) {
    galleryStore.clearSelection();
    selectionBeforeDrag = [];
  } else {
    selectionBeforeDrag = [...galleryStore.selectedIds];
  }
}

function onMouseMove(e) {
  if (!isDragging.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  dragCurrent.x = e.clientX - rect.left;
  dragCurrent.y = e.clientY - rect.top + containerRef.value.scrollTop;
  const inRect = getFilesInRect(
    dragStart.x,
    dragStart.y,
    dragCurrent.x,
    dragCurrent.y,
  );
  galleryStore.selectedIds = [...new Set([...selectionBeforeDrag, ...inRect])];
}

function onMouseUp() {
  if (isDragging.value) isDragging.value = false;
}

function onMouseLeave() {
  if (isDragging.value) isDragging.value = false;
}

const anchorIndex = ref(-1);

function onCardClick(e, file) {
  const clickedIndex = props.files.findIndex((f) => f.path === file.path);
  containerRef.value?.focus();

  if (galleryStore.selectionMode || e.ctrlKey || e.metaKey) {
    galleryStore.toggleSelection(file.path);
    anchorIndex.value = clickedIndex;
  } else if (e.shiftKey && galleryStore.selectedIds.length > 0) {
    if (anchorIndex.value === -1) {
      const lastPath = galleryStore.selectedIds[0];
      anchorIndex.value = props.files.findIndex((f) => f.path === lastPath);
    }
    if (anchorIndex.value >= 0 && clickedIndex >= 0) {
      const from = Math.min(anchorIndex.value, clickedIndex);
      const to = Math.max(anchorIndex.value, clickedIndex);
      galleryStore.selectedIds = props.files
        .slice(from, to + 1)
        .map((f) => f.path);
    }
  } else {
    galleryStore.selectedIds = [file.path];
    anchorIndex.value = clickedIndex;
  }
}

async function onCardDblClick(file) {
  const isFolder =
    file.is_dir === true ||
    file.file_type === "directory" ||
    file.is_directory === true;
  if (isFolder) {
    await navigationStore.navigateTo(file.path);
  } else {
    emit("openQuickLook", file);
  }
}

const contextMenuItems = computed(() => {
  const items = [];
  if (galleryStore.clipboard.paths.length > 0) {
    items.push({
      label: "Вставить",
      action: () => galleryStore.paste(navigationStore.currentPath),
    });
  }
  items.push({
    label: "Создать папку",
    action: () => createFolderInCurrentDir(),
  });
  items.push({
    label: "Выделить всё",
    action: () => galleryStore.selectAll(),
  });
  return items;
});

function handleContextMenu(e) {
  if (e.target.closest(".thumbnail-card")) return;
  contextMenuRef.value?.open(e.clientX, e.clientY);
}

async function createFolderInCurrentDir() {
  if (!navigationStore.currentPath) return;
  const separator = navigationStore.currentPath.includes("/") ? "/" : "\\";
  let name = "Новая папка";
  let counter = 1;
  const checkNameExists = (n) =>
    props.files.some((f) => f.name.toLowerCase() === n.toLowerCase());
  while (checkNameExists(name)) {
    counter++;
    name = `Новая папка (${counter})`;
  }
  const newPath = navigationStore.currentPath.endsWith(separator)
    ? navigationStore.currentPath + name
    : navigationStore.currentPath + separator + name;

  try {
    await invoke("mkdir_folder", { path: newPath });
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
      _newlyCreated: true,
    };
    navigationStore.folders = [
      newFolder,
      ...navigationStore.folders.filter((f) => f.path !== newPath),
    ];
    galleryStore.upsertFile(newFolder, { pinToTop: true });
    await nextTick();
    containerRef.value?.focus();
    galleryStore.renamingPath = newPath;
  } catch (e) {
    alert("Не удалось создать папку: " + e);
  }
}

function onGroupHeaderClick(e, row) {
  if (e.ctrlKey || e.metaKey) {
    const allSelected = row.filePaths.every((path) =>
      galleryStore.selectedIds.includes(path),
    );
    if (allSelected) {
      galleryStore.selectedIds = galleryStore.selectedIds.filter(
        (path) => !row.filePaths.includes(path),
      );
    } else {
      galleryStore.selectedIds = [
        ...new Set([...galleryStore.selectedIds, ...row.filePaths]),
      ];
    }
  } else {
    toggleGroupCollapse(row.title);
  }
}

function scrollToIndex(index) {
  const container = containerRef.value;
  if (!container || index < 0 || index >= props.files.length) return;
  const path = props.files[index]?.path;
  const el = container.querySelector(
    `.thumbnail-card[data-path="${CSS.escape(path)}"]`,
  );
  el?.scrollIntoView({ block: "nearest", inline: "nearest" });
}

function onKeyDown(e) {
  if (
    e.ctrlKey &&
    (e.code === "KeyA" ||
      e.key.toLowerCase() === "a" ||
      e.key === "ф" ||
      e.key === "Ф")
  ) {
    e.preventDefault();
    galleryStore.selectAll();
    return;
  }

  if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)) {
    e.preventDefault();
    if (props.files.length === 0) return;

    let currentIndex = -1;
    if (galleryStore.selectedIds.length > 0) {
      const lastSelectedPath =
        galleryStore.selectedIds[galleryStore.selectedIds.length - 1];
      currentIndex = props.files.findIndex((f) => f.path === lastSelectedPath);
    }
    if (currentIndex === -1) currentIndex = 0;

    const cols = Math.max(1, colsPerRow.value || 1);
    let nextIndex = currentIndex;
    if (e.key === "ArrowLeft") nextIndex = Math.max(0, currentIndex - 1);
    else if (e.key === "ArrowRight")
      nextIndex = Math.min(props.files.length - 1, currentIndex + 1);
    else if (e.key === "ArrowUp") nextIndex = Math.max(0, currentIndex - cols);
    else if (e.key === "ArrowDown")
      nextIndex = Math.min(props.files.length - 1, currentIndex + cols);

    if (e.shiftKey) {
      if (anchorIndex.value === -1) anchorIndex.value = currentIndex;
      const from = Math.min(anchorIndex.value, nextIndex);
      const to = Math.max(anchorIndex.value, nextIndex);
      galleryStore.selectedIds = props.files
        .slice(from, to + 1)
        .map((f) => f.path);
    } else {
      anchorIndex.value = nextIndex;
      galleryStore.selectedIds = [props.files[nextIndex].path];
    }
    scrollToIndex(nextIndex);
  } else if (e.key === " ") {
    e.preventDefault();
    if (galleryStore.selectedIds.length > 0) {
      const lastSelectedPath =
        galleryStore.selectedIds[galleryStore.selectedIds.length - 1];
      const file = props.files.find((f) => f.path === lastSelectedPath);
      if (file) onCardDblClick(file);
    }
  }
}

let resizeObserver = null;
onMounted(() => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth || 1200;
    containerHeight.value = containerRef.value.clientHeight || 800;
    scrollTop.value = containerRef.value.scrollTop || 0;
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerWidth.value = entry.contentRect.width;
        containerHeight.value = entry.contentRect.height;
      }
    });
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (resizeObserver) resizeObserver.disconnect();
});
</script>

<style scoped>
.virtual-scroll-gallery {
  position: relative;
  overflow-anchor: none;
  overflow-x: hidden;
  cursor: default;
}

.rubber-band {
  position: absolute;
  border: 1px solid rgba(59, 130, 246, 0.9);
  background: rgba(59, 130, 246, 0.15);
  pointer-events: none;
  z-index: 30;
}
</style>
