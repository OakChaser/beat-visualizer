<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/player";

const store = usePlayerStore();
const audioEl = ref<HTMLAudioElement | null>(null);

const currentTime = computed(() => formatTime(store.elapsed));
const totalTime = computed(() => formatTime(store.duration));

function formatTime(s: number): string {
  if (!isFinite(s) || s < 0) return "0:00";
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60);
  return `${m}:${sec.toString().padStart(2, "0")}`;
}

let animFrame: number | null = null;

function startTick() {
  if (animFrame !== null) return;
  const tick = () => {
    if (audioEl.value) {
      store.updateElapsed(audioEl.value.currentTime);
    }
    animFrame = requestAnimationFrame(tick);
  };
  animFrame = requestAnimationFrame(tick);
}

function stopTick() {
  if (animFrame !== null) {
    cancelAnimationFrame(animFrame);
    animFrame = null;
  }
}

async function playCurrent() {
  const audio = audioEl.value;
  const song = store.currentSong;
  if (!audio || !song) return;
  audio.src = convertFileSrc(song.path);
  audio.load();
  try {
    await audio.play();
  } catch (e) {
    console.error("Playback failed:", e);
    store.setPlayState("paused");
    return;
  }
  startTick();
  store.setPlayState("playing");
}

async function togglePlay() {
  const audio = audioEl.value;
  if (!audio) return;

  if (store.playState === "playing") {
    audio.pause();
    stopTick();
    store.setPlayState("paused");
  } else if (store.playState === "paused") {
    try {
      await audio.play();
    } catch (e) {
      console.error("Playback failed:", e);
      return;
    }
    startTick();
    store.setPlayState("playing");
  } else {
    await playCurrent();
  }
}

function skip(delta: number) {
  if (delta > 0) store.nextSong();
  else store.prevSong();
  void playCurrent();
}

function onProgressClick(e: MouseEvent) {
  const audio = audioEl.value;
  if (!audio || !audio.duration) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  if (rect.width === 0) return;
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  audio.currentTime = ratio * audio.duration;
  store.updateElapsed(audio.currentTime);
}

function onTimeUpdate() {
  const audio = audioEl.value;
  if (!audio) return;
  store.updateDuration(audio.duration || 0);
}

function onEnded() {
  store.nextSong();
  void playCurrent();
}

watch(
  () => store.currentIndex,
  () => {
    if (store.playState === "playing") void playCurrent();
  },
);

onMounted(() => {
  const audio = new Audio();
  audio.addEventListener("timeupdate", onTimeUpdate);
  audio.addEventListener("ended", onEnded);
  audioEl.value = audio;
});

onUnmounted(() => {
  stopTick();
  const audio = audioEl.value;
  if (!audio) return;
  audio.pause();
  audio.removeEventListener("timeupdate", onTimeUpdate);
  audio.removeEventListener("ended", onEnded);
  audio.src = "";
});
</script>

<template>
  <div class="controls">
    <button class="btn-icon" @click="skip(-1)" title="Previous">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
      </svg>
    </button>

    <button
      class="btn-play"
      @click="togglePlay"
      :title="store.playState === 'playing' ? 'Pause' : 'Play'"
    >
      <svg v-if="store.playState !== 'playing'" width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z"/>
      </svg>
      <svg v-else width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
      </svg>
    </button>

    <button class="btn-icon" @click="skip(1)" title="Next">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
      </svg>
    </button>
  </div>

  <div class="progress-bar" @click="onProgressClick">
    <div class="progress-fill" :style="{ width: store.progress * 100 + '%' }"></div>
  </div>

  <div class="time-display">
    <span>{{ currentTime }}</span>
    <span>{{ totalTime }}</span>
  </div>
</template>

<style scoped>
.controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-icon {
  background: none;
  border: none;
  color: #aaa;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s, background 0.15s;
}

.btn-icon:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}

.btn-play {
  background: #7c5cfc;
  border: none;
  color: #fff;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
}

.btn-play:hover {
  background: #6a4be0;
}

.btn-play:active {
  transform: scale(0.95);
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: #2a2a30;
  border-radius: 2px;
  cursor: pointer;
  margin: 10px 0 6px;
}

.progress-bar:hover {
  height: 6px;
}

.progress-fill {
  height: 100%;
  background: #7c5cfc;
  border-radius: 2px;
  transition: width 0.1s linear;
}

.time-display {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #555;
  font-variant-numeric: tabular-nums;
}
</style>