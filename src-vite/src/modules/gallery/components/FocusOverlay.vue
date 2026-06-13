<template>
  <div
    class="focus-overlay fixed inset-0 z-50 bg-base-300 flex select-none"
    @dragover.prevent
    @drop.prevent.stop="onDrop"
  >
    <!-- Левая панель: избранные папки -->
    <aside
      class="w-64 shrink-0 bg-base-200/95 border-r border-neutral/20 flex flex-col backdrop-blur-sm"
    >
      <div
        class="px-4 py-3 text-[10px] font-bold text-base-content/50 uppercase tracking-widest border-b border-neutral/15 flex items-center gap-2"
      >
        <span>⭐</span>
        <span>Избранное</span>
        <span class="ml-auto text-base-content/30 normal-case font-normal">
          {{ favoriteFolders.length }}
        </span>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar px-2 py-2 space-y-1">
        <div
          v-if="favoriteFolders.length === 0"
          class="px-3 py-6 text-xs text-base-content/40 italic text-center"
        >
          Добавьте папки в избранное,
          <br />
          затем перетаскивайте выбранные файлы сюда.
        </div>
        <button
          v-for="path in favoriteFolders"
          :key="path"
          class="group w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-xs font-semibold transition-all duration-150 border border-transparent text-left"
          :class="
            dragOverFavPath === path
              ? 'bg-primary/20 text-primary border-dashed border-primary/60'
              : 'text-base-content/75 hover:bg-base-100/60 hover:text-base-content hover:border-neutral/20'
          "
          @click="navigateTo(path)"
          @dragover.prevent="dragOverFavPath = path"
          @dragenter.prevent="dragOverFavPath = path"
          @dragleave="dragOverFavPath = ''"
          @drop.prevent.stop="handleFavDrop($event, path)"
          :title="path"
        >
          <span class="text-sm shrink-0">⭐</span>
          <span class="flex-1 truncate">{{ getFileName(path) }}</span>
        </button>
      </div>

      <div
        class="px-3 py-2 border-t border-neutral/15 text-[10px] text-base-content/40"
      >
        ESC — выйти из режима фокуса
      </div>
    </aside>

    <!-- Основная зона: галерея миниатюр -->
    <div class="flex-1 flex flex-col overflow-hidden bg-base-100">
      <div
        class="h-10 border-b border-neutral/15 bg-base-200/80 backdrop-blur flex items-center px-4 gap-3 text-xs text-base-content/70"
      >
        <span class="font-semibold">{{
          navigationStore.currentPath
            ? getFileName(navigationStore.currentPath)
            : "Главная"
        }}</span>
        <span class="text-base-content/30">·</span>
        <span>Элементов: {{ galleryStore.files.length }}</span>
        <span class="text-base-content/30">·</span>
        <span class="text-primary font-semibold">
          Выбрано: {{ galleryStore.selectedIds.length }}
        </span>
        <span class="ml-auto text-base-content/40 text-[10px]">
          Перетащите выделенные файлы в папку слева, чтобы скопировать
        </span>
      </div>

      <div class="flex-1 overflow-hidden">
        <VirtualScrollGallery
          :files="galleryStore.displayedFiles"
          :thumbnailSize="galleryStore.thumbnailSize"
          @openQuickLook="openQuickLook"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useNavigationStore } from "@/modules/navigation/store";
import { useGalleryStore } from "@/modules/gallery/store";
import { useConfigStore } from "@/stores/configStore";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import VirtualScrollGallery from "./VirtualScrollGallery.vue";

const emit = defineEmits(["open-quick-look", "exit-focus"]);

const navigationStore = useNavigationStore();
const galleryStore = useGalleryStore();
const configStore = useConfigStore();

const validFavoritePaths = ref([]);
const favoriteFolders = computed(() => validFavoritePaths.value);
const dragOverFavPath = ref("");

const getFileName = (path) => {
  if (!path) return "";
  const idx = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return idx >= 0 ? path.slice(idx + 1) : path;
};

const navigateTo = (path) => {
  navigationStore.navigateTo(path);
};

const openQuickLook = (file) => emit("open-quick-look", file);

const handleFavDrop = async (e, destPath) => {
  dragOverFavPath.value = "";
  try {
    const data = e.dataTransfer.getData("text/plain");
    if (!data) return;
    const paths = JSON.parse(data);
    if (!Array.isArray(paths) || paths.length === 0) return;

    let success = 0;
    for (const src of paths) {
      if (src === destPath) continue;
      const lastSlash = Math.max(src.lastIndexOf("\\"), src.lastIndexOf("/"));
      const fileName = lastSlash >= 0 ? src.slice(lastSlash + 1) : src;
      const dest = `${destPath}\\${fileName}`;
      try {
        await invoke("cross_copy", { src, dest });
        success++;
      } catch (err) {
        console.error("Focus copy failed:", err);
      }
    }
    if (success > 0 && isTauri()) {
      // Прячем окно в панель задач, если пользователь явно не отказался.
      try {
        await getCurrentWindow().minimize();
      } catch {}
    }
  } catch (err) {
    console.error("Focus drop handler failed:", err);
  }
};

const onDrop = (e) => {
  // Drop в основной зоне (не на папку) — игнорируем, чтобы не закрывало режим.
  e.preventDefault();
};

async function refreshFavorites() {
  const favorites = Array.isArray(configStore.settings.favorites)
    ? [...configStore.settings.favorites]
    : [];

  if (!isTauri()) {
    validFavoritePaths.value = favorites;
    return;
  }

  const checks = await Promise.all(
    favorites.map(async (path) => {
      try {
        await invoke("get_file_entry", { path });
        return path;
      } catch {
        return null;
      }
    }),
  );

  const valid = checks.filter(Boolean);
  validFavoritePaths.value = valid;

  if (valid.length !== favorites.length) {
    configStore.settings.favorites = valid;
  }
}

// ESC → выход
const onKey = (e) => {
  if (e.key === "Escape" && galleryStore.focusMode) {
    e.preventDefault();
    e.stopPropagation();
    galleryStore.exitFocusMode();
    emit("exit-focus");
  }
};

onMounted(() => {
  refreshFavorites();
  window.addEventListener("keydown", onKey, { capture: true });
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKey, { capture: true });
});
</script>

<style scoped>
.focus-overlay {
  /* Поверх основного layout, но не ломаем текущий selection. */
  animation: focusFade 160ms ease-out;
}
@keyframes focusFade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
</style>
