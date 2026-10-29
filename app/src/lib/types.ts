// Mirrors src-tauri/src/model.rs exactly. Keep both in sync.

export interface DateRef {
  label: string;
  date: string;
  kind: "deadline" | "expiry" | "event" | "taken" | "mentioned";
}

export interface SecretRef {
  kind: "password" | "id_number" | "card_number" | "location";
  preview: string;
  page: number | null;
  bbox: number[] | null;
}

export interface DeadlineItem {
  fileId: string;
  path: string;
  name: string;
  label: string;
  date: string;
  daysLeft: number;
}

export interface SecretItem {
  fileId: string;
  path: string;
  name: string;
  kind: string;
  preview: string;
  hasGps: boolean;
  redactable: boolean;
}

export interface RestItem {
  fileId: string;
  path: string;
  name: string;
  size: number;
  ageDays: number;
  dead: number;
  deadReason: string | null;
}

export interface AppState {
  deadlines: DeadlineItem[];
  secrets: SecretItem[];
  rest: RestItem[];
  filesTotal: number;
  restFreed: number;
  roots: string[];
}

export interface ScanProgress {
  done: number;
  total: number;
  current: string;
  stage: "reading" | "model" | "done" | null;
}

export interface SearchHit {
  fileId: string;
  path: string;
  name: string;
  category: string;
  score: number;
}

export interface ActionOk {
  message: string;
  undoable: boolean;
  path: string | null;
  undoFileIds: string[];
  undoPaths: string[];
}

