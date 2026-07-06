<template>
  <div
    ref="listContainerRef"
    class="list-view-container h-full overflow-y-auto overflow-x-hidden focus:outline-none px-4 py-2"
    tabindex="0"
    @keydown="onKeyDown"
    @contextmenu.prevent.stop="handleContainerContextMenu"
    style="user-select: none;"
  >
    <div class="flex flex-col gap-1">
      <div
        v-for="file in files"
        :key="file.path"
        class="list-row flex items-center gap-3 px-3 py-1.5 rounded-lg border border-transparent hover:bg-base-200/60 cursor-pointer transition-all duration-150"
        :class="{
          'bg-primary/10 border-primary/45 shadow-sm': selectedIds.includes(file.path)
        }"
        @click.stop="onItemClick($event, file)"
        @dblclick.stop="onItemDblClick(file)"
        @contextmenu.prevent.stop="handleItemContextMenu($event, file)"
      >
        <FileIcon :file="file" :size="24" />
        
        <div v-if="galleryStore.renamingPath === file.path" class="flex-1">
          <input
            ref="renameInputRef"
            v-model="renameText"
            type="text"
            class="rename-input-el input input-xs input-bordered w-full text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary bg-base-100 text-base-content"
            @keydown.enter.stop="saveRename(file)"
            @keydown.esc.stop="cancelRename"
            @blur="saveRename(file)"
            @click.stop
          />
        </div>
        <span v-else class="text-sm font-medium truncate text-base-content/90 flex-1" :title="file.name">
          {{ file.name }}
        </span>
      </div>
    </div>

    <!-- Message box for delete confirmation -->
    <MessageBox
      v-if="showConfirm"
      title="Удаление"
      :message="confirmMsg"
      :warningOk="true"
      checkboxText="Больше не спрашивать"
      :checkboxChecked="skipDeleteCheckboxVal"
      @checkbox-change="(val) => (skipDeleteCheckboxVal = val)"
      @ok="confirmDeletion"
      @cancel="showConfirm = false"
    />

    <!-- Context menu -->
    <ContextMenu
      ref="contextMenuRef"
      :menuItems="activeContextMenuItems"
      :smallIcon="true"
      style="display: none;"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from "vue";
import { useGalleryStore } from "../store";
import { useNavigationStore } from "../../navigation/store";
import { useConfigStore } from "@/stores/configStore";
import FileIcon from "./FileIcon.vue";
import ContextMenu from "@/components/ContextMenu.vue";
import MessageBox from "@/components/MessageBox.vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps({
  files: { type: Array, default: () => [] }
});

const emit = defineEmits(["openQuickLook"]);

const galleryStore = useGalleryStore();
const navigationStore = useNavigationStore();
const configStore = useConfigStore();

const selectedIds = computed(() => galleryStore.selectedIds);

const listContainerRef = ref(null);
const contextMenuRef = ref(null);
const activeFile = ref(null);

const renameText = ref("");
const renameInputRef = ref(null);

// selection helper
const anchorIndex = ref(-1);

function onItemClick(e, file) {
  const clickedIndex = props.files.findIndex((f) => f.path === file.path);
  listContainerRef.value?.focus();
  const isMultiSelect = galleryStore.selectionMode || e.ctrlKey || e.metaKey;

  if (e.shiftKey) {
    if (anchorIndex.value === -1) {
      if (galleryStore.selectedIds.length > 0) {
        const anchorPath = galleryStore.selectedIds[0];
        anchorIndex.value = props.files.findIndex((f) => f.path === anchorPath);
      }
      if (anchorIndex.value === -1) {
        anchorIndex.value = clickedIndex;
      }
    }
    if (anchorIndex.value >= 0 && clickedIndex >= 0) {
      const from = Math.min(anchorIndex.value, clickedIndex);
      const to = Math.max(anchorIndex.value, clickedIndex);
      const rangePaths = props.files.slice(from, to + 1).map((f) => f.path);
      if (isMultiSelect) {
        galleryStore.selectedIds = [...new Set([...galleryStore.selectedIds, ...rangePaths])];
      } else {
        galleryStore.selectedIds = rangePaths;
      }
    }
  } else {
    if (isMultiSelect) {
      galleryStore.toggleSelection(file.path);
    } else {
      galleryStore.selectedIds = [file.path];
    }
    anchorIndex.value = clickedIndex;
  }
}

async function onItemDblClick(file) {
  const isFolder = file.is_dir === true || file.file_type === "directory" || file.is_directory === true;
  if (isFolder) {
    await navigationStore.navigateTo(file.path);
  } else {
    emit("openQuickLook", file);
  }
}

// Rename functionality
watch(() => galleryStore.renamingPath, (newPath) => {
  if (newPath) {
    const file = props.files.find(f => f.path === newPath);
    if (file) {
      renameText.value = file.name;
      nextTick(() => {
        const input = renameInputRef.value?.[0] || document.querySelector(".rename-input-el");
        if (input) {
          input.focus();
          const dotIndex = file.name.lastIndexOf(".");
          const isDir = file.is_dir === true || file.file_type === "directory" || file.is_directory === true;
          if (dotIndex > 0 && !isDir) {
            input.setSelectionRange(0, dotIndex);
          } else {
            input.select();
          }
        }
      });
    }
  }
});

async function saveRename(file) {
  if (galleryStore.renamingPath !== file.path) return;
  galleryStore.renamingPath = null;
  const newName = renameText.value.trim();
  if (!newName || newName === file.name) return;

  const lastSlash = Math.max(file.path.lastIndexOf("\\"), file.path.lastIndexOf("/"));
  const parentPath = lastSlash !== -1 ? file.path.substring(0, lastSlash) : "";
  const separator = file.path.includes("/") ? "/" : "\\";
  const newPath = parentPath ? `${parentPath}${separator}${newName}` : newName;

  try {
    await invoke("cross_move", { src: file.path, dest: newPath });
    await navigationStore.refresh();
  } catch (e) {
    alert("Ошибка переименования: " + e);
  }
}

function cancelRename() {
  galleryStore.renamingPath = null;
}

// Deletion
const showConfirm = ref(false);
const confirmMsg = ref("");
const skipDeleteCheckboxVal = ref(false);
const pathsToDelete = ref([]);

function deleteItem(file) {
  const paths = galleryStore.selectedIds.includes(file.path)
    ? [...galleryStore.selectedIds]
    : [file.path];
  
  pathsToDelete.value = paths;
  
  if (configStore.settings.skipDeleteConfirmation) {
    confirmDeletion();
  } else {
    const count = paths.length;
    if (count === 1) {
      confirmMsg.value = `Вы действительно хотите удалить "${file.name}"?`;
    } else {
      confirmMsg.value = `Вы действительно хотите удалить выбранные элементы (${count})?`;
    }
    showConfirm.value = true;
  }
}

async function confirmDeletion() {
  showConfirm.value = false;
  if (skipDeleteCheckboxVal.value) {
    configStore.settings.skipDeleteConfirmation = true;
  }
  try {
    await galleryStore.deleteFiles(pathsToDelete.value);
  } catch (err) {
    alert("Ошибка при удалении: " + err);
  }
}

// Clipboard
function copyItem(file) {
  const paths = galleryStore.selectedIds.includes(file.path)
    ? [...galleryStore.selectedIds]
    : [file.path];
  galleryStore.setClipboard("copy", paths);
}

function cutItem(file) {
  const paths = galleryStore.selectedIds.includes(file.path)
    ? [...galleryStore.selectedIds]
    : [file.path];
  galleryStore.setClipboard("cut", paths);
}

// Folder icon customization
const FOLDER_ICON_URLS_DICT = {
  "01.ico": "01.ico",
  "02.ico": "02.ico",
  "03.ico": "03.ico",
  "04.ico": "04.ico",
  "05.ico": "05.ico",
  "06.ico": "06.ico",
  "07.ico": "07.ico",
  "08.ico": "08.ico",
  "09.ico": "09.ico",
  "10.ico": "10.ico",
  "11.ico": "11.ico",
  "12.ico": "12.ico",
  "14.ico": "14.ico",
  "15.ico": "15.ico",
  "I1.ico": "I1.ico",
};

function setFolderIcon(file, iconName) {
  configStore.setFolderIcon(file.path, iconName);
}

// Context Menu items definition
const activeContextMenuItems = computed(() => {
  const file = activeFile.value;
  if (!file) return [];
  const isFolder = file.is_dir === true || file.file_type === "directory" || file.is_directory === true;

  const items = [
    { label: "Переименовать", action: () => { galleryStore.renamingPath = file.path; } },
    { label: "Копировать", action: () => copyItem(file) },
    { label: "Вырезать", action: () => cutItem(file) },
    { label: "Удалить", action: () => deleteItem(file) },
  ];

  if (galleryStore.clipboard.paths.length > 0) {
    items.push({
      label: "Вставить",
      action: () => galleryStore.paste(navigationStore.currentPath),
    });
  }

  if (!isFolder) {
    const inStack = galleryStore.isInCompareStack(file.path);
    if (inStack) {
      items.push({ separator: true });
      items.push({
        label: "Убрать из сравнения",
        action: () => galleryStore.removeFromCompare(file.path),
      });
    } else if (galleryStore.canAddMoreToCompare) {
      items.push({ separator: true });
      items.push({
        label: "Добавить к сравнению",
        action: () => galleryStore.addToCompare(file),
      });
    }
  }

  if (isFolder) {
    const isFav = configStore.settings.favorites?.includes(file.path);
    items.push({
      label: isFav ? "Удалить из избранного" : "Добавить в избранное",
      action: () => configStore.toggleFavorite(file.path),
    });
    
    // Add folder recoloring children
    items.push({ separator: true });
    items.push({
      label: "Перекрасить папку",
      grid: true,
      children: Object.keys(FOLDER_ICON_URLS_DICT).map(iconName => ({
        // We will pass tooltip and iconName as action
        tooltip: iconName === "14.ico" ? "По умолчанию" : iconName === "I1.ico" ? "Важная" : `Папка ${iconName.split('.')[0]}`,
        action: () => setFolderIcon(file, iconName)
      }))
    });
  }
  return items;
});

function handleItemContextMenu(e, file) {
  if (!galleryStore.selectedIds.includes(file.path)) {
    galleryStore.selectedIds = [file.path];
  }
  activeFile.value = file;
  contextMenuRef.value?.open(e.clientX, e.clientY);
}

const containerContextMenuItems = computed(() => {
  const items = [];
  if (galleryStore.clipboard.paths.length > 0) {
    items.push({
      label: "Вставить",
      action: () => galleryStore.paste(navigationStore.currentPath),
    });
  }
  items.push({
    label: "Выделить всё",
    action: () => galleryStore.selectAll(),
  });
  return items;
});

function handleContainerContextMenu(e) {
  if (e.target.closest(".list-row")) return;
  activeFile.value = null;
  contextMenuRef.value?.open(e.clientX, e.clientY);
}

function onKeyDown(e) {
  if (e.key === "Delete") {
    if (galleryStore.selectedIds.length > 0) {
      const firstSelected = props.files.find(f => f.path === galleryStore.selectedIds[0]);
      if (firstSelected) deleteItem(firstSelected);
    }
  }
}
</script>

<style scoped>
.list-view-container {
  scrollbar-gutter: stable;
}
.list-row {
  user-select: none;
}
</style>
