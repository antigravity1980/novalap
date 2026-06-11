/**
 * explorerThumbnailsCache.js
 *
 * Singleton in-memory cache for Explorer Mode thumbnail URLs.
 *
 * WHY: Virtual scroll recycles Vue components during scrolling. When a card
 * scrolls off screen and back on, Vue unmounts / remounts the component,
 * which resets the local `thumbnailUrl` ref and triggers a new IPC call to
 * `get_explorer_thumbnail`. This makes thumbnails disappear for ~150ms+ on
 * every scroll.
 *
 * HOW: We store the resolved data-URL (or asset URL) for each file path in a
 * plain JS Map. On next render the card finds its cached URL immediately and
 * sets `thumbnailUrl` synchronously — zero flicker, zero IPC round-trip.
 *
 * MEMORY: The map is cleared whenever the user navigates to a different
 * directory, keeping the footprint bounded to the current folder contents.
 */

/** @type {Map<string, string>} path → data-URL / asset-URL */
const _cache = new Map();

/**
 * Retrieve a cached thumbnail URL for the given file path.
 * @param {string} filePath
 * @returns {string|undefined}
 */
export function getCachedThumbnail(filePath) {
  return _cache.get(filePath);
}

/**
 * Store a resolved thumbnail URL for the given file path.
 * @param {string} filePath
 * @param {string} url
 */
export function setCachedThumbnail(filePath, url) {
  _cache.set(filePath, url);
}

/**
 * Clear cached thumbnail URL for a single file path.
 * @param {string} filePath
 */
export function clearCachedThumbnail(filePath) {
  if (!filePath) return;
  _cache.delete(filePath);
}

/**
 * Clear cached thumbnail URL by full cache key.
 * @param {string} cacheKey
 */
export function clearCachedThumbnailByKey(cacheKey) {
  if (!cacheKey) return;
  _cache.delete(cacheKey);
}

/**
 * Clear all cached thumbnail URLs.
 * Call this when the user navigates to a new directory.
 */
export function clearThumbnailCache() {
  _cache.clear();
}
