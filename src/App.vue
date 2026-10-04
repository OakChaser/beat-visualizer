<script setup lang="ts">
import { onMounted } from "vue";
import { usePlayerStore } from "./stores/player";
import Playlist from "./components/Playlist.vue";
import PlayerControls from "./components/PlayerControls.vue";
import BeatVisualizer from "./components/BeatVisualizer.vue";

const store = usePlayerStore();

onMounted(async () => {
  await store.refreshAnalyzerStatus();
  await store.restoreMusicDir();
  if (store.songs.length > 0) {
    store.selectSong(0);
  }
});
</script>

<template>
  <div class="app">
    <Playlist />
    <div class="main">
      <BeatVisualizer />
      <div class="bottom-bar">
        <div class="song-info">
          <span class="now-label">NOW PLAYING</span>
          <span class="now-title">{{ store.currentSong?.name || "---" }}</span>
        </div>
        <PlayerControls />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  width: 100%;
  height: 100vh;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.bottom-bar {
  padding: 16px 24px 20px;
  background: #12121a;
  border-top: 1px solid #2a2a30;
}

.song-info {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 12px;
}

.now-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: #7c5cfc;
}

.now-title {
  font-size: 14px;
  color: #ccc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>