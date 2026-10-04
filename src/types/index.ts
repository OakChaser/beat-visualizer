export interface SongInfo {
  name: string;
  path: string;
}

export interface BeatInfo {
  beats: number[];
  downbeats: number[];
  bpm: number | null;
}

export interface AnalyzerStatus {
  ready: boolean;
  modelDir: string | null;
  error: string | null;
}

export type PlayState = "idle" | "playing" | "paused";

export type BeatStatus = "idle" | "loading" | "done" | "error";