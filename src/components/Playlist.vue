<script setup lang="ts">
import { computed } from "vue";
import { usePlayerStore } from "../stores/player";

const store = usePlayerStore();
const activeIndex = computed(() => store.currentIndex);
const folderName = computed(() => {
  const dir = store.musicDir;
  if (!dir) return null;
  return dir.split(/[\\/]/).filter(Boolean).pop() ?? dir;
});
</script>

<template>
  <div class="playlist">
    <div class="playlist-header">
      <span>PLAYLIST</span>
      <button
        class="btn-folder"
        title="Choose music folder"
        @click="store.pickMusicDir()"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z"
          />
        </svg>
      </button>
    </div>

    <div v-if="folderName" class="playlist-folder" :title="store.musicDir">
      {{ folderName }}
    </div>

    <div class="playlist-list">
      <div
        v-for="(song, i) in store.songs"
        :key="song.path"
        class="playlist-item"
        :class="{ active: i === activeIndex }"
        @click="store.selectSong(i)"
      >
        <span class="playlist-index">{{ i + 1 }}</span>
        <span class="playlist-name">{{ song.name }}</span>
      </div>
      <div v-if="store.songs.length === 0" class="playlist-empty">
        No songs found
      </div>
    </div>
  </div>
</template>

<style scoped>
.playlist {
  width: 240px;
  min-width: 240px;
  background: #16161a;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #2a2a30;
}

.playlist-header {
  padding: 20px 16px 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #72727a;
}

.btn-folder {
  background: none;
  border: none;
  color: #5c5c66;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  transition: color 0.15s, background 0.15s;
}

.btn-folder:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}

.playlist-folder {
  padding: 0 16px 12px;
  font-size: 11px;
  color: #4a4a52;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-list {
  flex: 1;
  overflow-y: auto;
}

.playlist-item {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  cursor: pointer;
  gap: 12px;
  transition: background 0.15s;
}

.playlist-item:hover {
  background: #1e1e24;
}

.playlist-item.active {
  background: #2a2a35;
}

.playlist-item.active .playlist-name {
  color: #fff;
}

.playlist-item.active .playlist-index {
  color: #7c5cfc;
}

.playlist-index {
  font-size: 12px;
  color: #555;
  width: 20px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.playlist-name {
  font-size: 13px;
  color: #aaa;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-empty {
  padding: 40px 16px;
  text-align: center;
  color: #555;
  font-size: 13px;
}
</style>