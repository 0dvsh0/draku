// Typed IPC. Every Draku capability the UI has is one of these.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { ActionOk, AppState, ScanProgress, SearchHit } from "./types";

export const api = {
  getState: () => invoke<AppState>("get_state"),

  startScan: (paths: string[]) => invoke<ActionOk>("start_scan", { paths }),

  cancelScan: () => invoke<void>("cancel_scan"),

  search: (query: string) => invoke<SearchHit[]>("search", { query }),

  addToCalendar: (fileId: string) => invoke<ActionOk>("add_to_calendar", { fileId }),

  makeSafeCopy: (fileId: string) => invoke<ActionOk>("make_safe_copy", { fileId }),

  setAside: (fileId: string) => invoke<ActionOk>("set_aside", { fileId }),

  setAsideMany: (fileIds: string[]) => invoke<ActionOk>("set_aside_many", { fileIds }),

  restore: (fileId: string) => invoke<ActionOk>("restore", { fileId }),

  restoreMany: (fileIds: string[]) => invoke<ActionOk>("restore_many", { fileIds }),

  undoGenerated: (paths: string[]) => invoke<void>("undo_generated", { paths }),

  openFolderDialog: () =>
    open({ directory: true, multiple: false, title: "Give Draku a folder" }),

  reveal: (path: string) => revealItemInDir(path),
};

export const events = {
  onScanProgress: (cb: (p: ScanProgress) => void) =>
    listen<ScanProgress>("scan://progress", (e) => cb(e.payload)),
  onScanFinished: (cb: () => void) => listen("scan://finished", cb),
  onNotice: (cb: (n: { level: string; message: string }) => void) =>
    listen<{ level: string; message: string }>("app://notice", (e) => cb(e.payload)),
  onDroppedFiles: (cb: (paths: string[]) => void) => {
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "drop" && e.payload.paths.length > 0) {
        cb(e.payload.paths.filter((p) => !p.includes(".draku")));
      }
    });
    return un;
  },
};
