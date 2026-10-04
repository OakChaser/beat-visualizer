import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AnalyzerStatus,
  BeatInfo,
  BeatStatus,
  PlayState,
  SongInfo,
} from "../types";

const MUSIC_DIR_STORAGE_KEY = "beat-visualizer.musicDir";

export const usePlayerStore = defineStore("player", () => {
  const songs = ref<SongInfo[]>([]);
  const currentIndex = ref(-1);
  const playState = ref<PlayState>("idle");
  const elapsed = ref(0);
  const duration = ref(0);
  const beatInfo = ref<BeatInfo | null>(null);
  const beatStatus = ref<BeatStatus>("idle");
  const beatError = ref<string | null>(null);
  const musicDir = ref("");
  const analyzer = ref<AnalyzerStatus>({
    ready: false,
    modelDir: null,
    error: null,
  });

  const currentSong = computed(() =>
    currentIndex.value >= 0 ? songs.value[currentIndex.value] : null,
  );

  const progress = computed(() =>
    duration.value > 0 ? elapsed.value / duration.value : 0,
  );

  /** Guards against out-of-order `analyze_beat` replies when the user clicks fast. */
  let analysisToken = 0;

  async function refreshAnalyzerStatus() {
    try {
      analyzer.value = await invoke<AnalyzerStatus>("analyzer_status");
    } catch (e) {
      analyzer.value = {
        ready: false,
        modelDir: null,
        error: String(e),
      };
    }
  }

  async function loadMusicDir(dir: string) {
    musicDir.value = dir;
    try {
      songs.value = await invoke<SongInfo[]>("list_songs", { musicDir: dir });
      localStorage.setItem(MUSIC_DIR_STORAGE_KEY, dir);
      return true;
    } catch (e) {
      console.error("Failed to list songs:", e);
      songs.value = [];
      return false;
    }
  }

  /** Reopens the folder chosen on a previous run, falling back to the picker. */
  async function restoreMusicDir() {
    const saved = localStorage.getItem(MUSIC_DIR_STORAGE_KEY);
    if (saved && (await loadMusicDir(saved))) return true;
    return pickMusicDir();
  }

  async function pickMusicDir() {
    const picked = await open({
      directory: true,
      multiple: false,
      title: "Choose a music folder",
    });
    if (typeof picked !== "string") return false;
    const ok = await loadMusicDir(picked);
    if (!ok) await pickMusicDir();
    return ok;
  }

  async function analyzeCurrent() {
    const song = currentSong.value;
    const token = ++analysisToken;

    if (!song) {
      beatInfo.value = null;
      beatStatus.value = "idle";
      beatError.value = null;
      return;
    }
    if (!analyzer.value.ready) {
      beatStatus.value = "error";
      beatError.value = analyzer.value.error ?? "Beat analysis is unavailable";
      return;
    }

    beatStatus.value = "loading";
    beatError.value = null;
    try {
      const info = await invoke<BeatInfo>("analyze_beat", { path: song.path });
      if (token !== analysisToken) return;
      beatInfo.value = info;
      beatStatus.value = "done";
    } catch (e) {
      if (token !== analysisToken) return;
      beatStatus.value = "error";
      beatError.value = String(e);
    }
  }

  function selectSong(index: number) {
    if (index < 0 || index >= songs.value.length) return;
    currentIndex.value = index;
    elapsed.value = 0;
    duration.value = 0;
    playState.value = "playing";
    void analyzeCurrent();
  }

  function nextSong() {
    if (songs.value.length === 0) return;
    selectSong((currentIndex.value + 1) % songs.value.length);
  }

  function prevSong() {
    if (songs.value.length === 0) return;
    selectSong(
      currentIndex.value <= 0 ? songs.value.length - 1 : currentIndex.value - 1,
    );
  }

  function updateElapsed(t: number) {
    elapsed.value = t;
  }

  function updateDuration(d: number) {
    duration.value = d;
  }

  function setPlayState(s: PlayState) {
    playState.value = s;
  }

  return {
    songs,
    currentIndex,
    currentSong,
    playState,
    elapsed,
    duration,
    progress,
    beatInfo,
    beatStatus,
    beatError,
    musicDir,
    analyzer,
    refreshAnalyzerStatus,
    restoreMusicDir,
    pickMusicDir,
    analyzeCurrent,
    selectSong,
    nextSong,
    prevSong,
    updateElapsed,
    updateDuration,
    setPlayState,
  };
});