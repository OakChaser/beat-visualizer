<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { gsap } from "gsap";
import { usePlayerStore } from "../stores/player";

const store = usePlayerStore();

const titleRef = ref<HTMLDivElement | null>(null);
const leftLight = ref<HTMLDivElement | null>(null);
const rightLight = ref<HTMLDivElement | null>(null);

// Track last beat/downbeat to detect new ones
let prevBeatIdx = -1;
let prevDownbeatIdx = -1;

function findNearestBeatIndex(elapsed: number, beats: number[]): number {
  for (let i = beats.length - 1; i >= 0; i--) {
    if (beats[i] <= elapsed + 0.05) return i;
  }
  return -1;
}

function triggerBeatAnimation() {
  if (!titleRef.value) return;
  gsap.killTweensOf(titleRef.value);
  gsap.fromTo(
    titleRef.value,
    { scale: 1.0, opacity: 0.85 },
    { scale: 1.25, opacity: 1, duration: 0.06, ease: "power2.out",
      onComplete: () => {
        gsap.to(titleRef.value, { scale: 1.0, opacity: 0.85, duration: 0.35, ease: "power2.out" });
      }
    }
  );
}

function triggerDownbeatAnimation() {
  if (!leftLight.value || !rightLight.value) return;
  gsap.killTweensOf(leftLight.value);
  gsap.killTweensOf(rightLight.value);
  const tl = gsap.timeline();

  tl.fromTo(
    leftLight.value,
    { opacity: 0, x: 0, scaleX: 1.0 },
    { opacity: 1, x: 0, scaleX: 1.5, duration: 0.08, ease: "power2.out" }
  );
  tl.to(
    leftLight.value,
    { opacity: 0, x: -60, scaleX: 2.5, duration: 1.0, ease: "power3.out" }
  );

  tl.fromTo(
    rightLight.value,
    { opacity: 0, x: 0, scaleX: 1.0 },
    { opacity: 1, x: 0, scaleX: 1.5, duration: 0.08, ease: "power2.out" },
    0
  );
  tl.to(
    rightLight.value,
    { opacity: 0, x: 60, scaleX: 2.5, duration: 1.0, ease: "power3.out" },
    0
  );
}

let rafId: number | null = null;

function tick() {
  const info = store.beatInfo;
  if (!info || store.playState !== "playing") {
    rafId = requestAnimationFrame(tick);
    return;
  }

  const t = store.elapsed;

  // Check beats
  const beatIdx = findNearestBeatIndex(t, info.beats);
  if (beatIdx >= 0 && beatIdx !== prevBeatIdx) {
    prevBeatIdx = beatIdx;
    triggerBeatAnimation();
  }

  // Check downbeats
  const downbeatIdx = findNearestBeatIndex(t, info.downbeats);
  if (downbeatIdx >= 0 && downbeatIdx !== prevDownbeatIdx) {
    prevDownbeatIdx = downbeatIdx;
    triggerDownbeatAnimation();
  }

  rafId = requestAnimationFrame(tick);
}

onMounted(() => {
  rafId = requestAnimationFrame(tick);
});

onUnmounted(() => {
  if (rafId !== null) cancelAnimationFrame(rafId);
});

// Reset tracking when song changes
watch(
  () => store.currentIndex,
  () => {
    prevBeatIdx = -1;
    prevDownbeatIdx = -1;
  }
);
</script>

<template>
  <div class="visualizer">
    <!-- Side lights -->
    <div class="side-light left" ref="leftLight"></div>
    <div class="side-light right" ref="rightLight"></div>

    <!-- Center content -->
    <div class="center-content">
      <div class="song-title" ref="titleRef">
        {{ store.currentSong?.name || "Beat Visualizer" }}
      </div>

      <div class="beat-status">
        <span v-if="!store.analyzer.ready" class="status error">
          Beat analysis unavailable — run <code>scripts/fetch-models.sh</code>
        </span>

        <span v-else-if="store.beatStatus === 'loading'" class="status loading">
          <span class="spinner"></span> Analyzing beat...
        </span>

        <span
          v-else-if="store.beatStatus === 'done' && store.beatInfo"
          class="status done"
        >
          <template v-if="store.beatInfo.bpm">
            {{ store.beatInfo.bpm.toFixed(1) }} BPM ·
          </template>
          Beats: {{ store.beatInfo.beats.length }} | Downbeats:
          {{ store.beatInfo.downbeats.length }}
        </span>

        <span
          v-else-if="store.beatStatus === 'error'"
          class="status error"
          :title="store.beatError ?? ''"
        >
          Beat analysis failed
        </span>
      </div>
    </div>

    <!-- Beat pulse background -->
    <div class="pulse-bg"></div>
  </div>
</template>

<style scoped>
.visualizer {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: radial-gradient(ellipse at center, #1a1a24 0%, #0f0f14 70%);
}

.side-light {
  position: absolute;
  top: 5%;
  width: 6px;
  height: 90%;
  border-radius: 3px;
  opacity: 0;
  pointer-events: none;
  transform-origin: center;
}

.side-light.left {
  left: 30px;
  background: linear-gradient(
    180deg,
    transparent 0%,
    rgba(124, 92, 252, 0.6) 20%,
    rgba(140, 100, 255, 0.95) 45%,
    rgba(160, 120, 255, 1) 50%,
    rgba(140, 100, 255, 0.95) 55%,
    rgba(124, 92, 252, 0.6) 80%,
    transparent 100%
  );
  filter: blur(12px);
  box-shadow: 0 0 30px 10px rgba(124, 92, 252, 0.5),
              0 0 60px 20px rgba(100, 70, 230, 0.3);
}

.side-light.right {
  right: 30px;
  background: linear-gradient(
    180deg,
    transparent 0%,
    rgba(124, 92, 252, 0.6) 20%,
    rgba(140, 100, 255, 0.95) 45%,
    rgba(160, 120, 255, 1) 50%,
    rgba(140, 100, 255, 0.95) 55%,
    rgba(124, 92, 252, 0.6) 80%,
    transparent 100%
  );
  filter: blur(12px);
  box-shadow: 0 0 30px 10px rgba(124, 92, 252, 0.5),
              0 0 60px 20px rgba(100, 70, 230, 0.3);
}

.center-content {
  position: relative;
  z-index: 2;
  text-align: center;
}

.song-title {
  font-size: 36px;
  font-weight: 700;
  color: #e0e0e5;
  letter-spacing: 2px;
  text-shadow: 0 0 40px rgba(124, 92, 252, 0.3);
  transform-origin: center;
  user-select: none;
}

.pulse-bg {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at center, rgba(124, 92, 252, 0.05) 0%, transparent 60%);
  pointer-events: none;
}

.beat-status {
  margin-top: 12px;
  font-size: 13px;
  letter-spacing: 1px;
}

.status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.status.loading {
  color: #7c5cfc;
}

.status.done {
  color: #5a5a6a;
}

.status.error {
  color: #e05555;
}

.status code {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  color: #c98d8d;
}

.spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid rgba(124, 92, 252, 0.3);
  border-top-color: #7c5cfc;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
