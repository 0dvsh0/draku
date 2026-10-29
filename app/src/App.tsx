import { useCallback, useEffect, useRef, useState } from "react";
import { api, events } from "./lib/api";
import type { AppState, ScanProgress } from "./lib/types";
import { Empty } from "./components/Empty";
import { Scanning } from "./components/Scanning";
import { OneScreen } from "./components/OneScreen";
import { Toast, Toasts } from "./components/Toasts";
import { Eye } from "./components/Eye";

type Phase = "empty" | "scanning" | "ready";

export default function App() {
  const [phase, setPhase] = useState<Phase>("empty");
  const [state, setState] = useState<AppState | null>(null);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [dragging, setDragging] = useState(false);
  const toastId = useRef(0);

  // ---- events -----------------------------------------------------------
  useEffect(() => {
    const un1 = events.onScanProgress((p) => {
      setProgress(p);
      if (p.stage === "reading") setPhase("scanning");
    });
    const un2 = events.onScanFinished(async () => {
      setProgress(null);
      const s = await refreshState();
      setPhase(s && s.filesTotal > 0 ? "ready" : "empty");
    });
    const un3 = events.onNotice((n) => pushToast(n.message));
    const un4 = events.onDroppedFiles(async (paths) => {
      if (paths.length === 0) return;
      const label =
        paths.length === 1
          ? paths[0].split(/[\\/]/).pop()
          : `${paths.length} items`;
      pushToast(`Reading ${label}`);
      setPhase("scanning");
      try {
        await api.startScan(paths);
      } catch (err) {
        pushToast(String(err));
        const s = await refreshState();
        setPhase(s && s.filesTotal > 0 ? "ready" : "empty");
      }
    });
    return () => {
      un1.then((f) => f());
      un2.then((f) => f());
      un3.then((f) => f());
      un4.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const refreshState = useCallback(async (): Promise<AppState | null> => {
    try {
      const s = await api.getState();
      setState(s);
      return s;
    } catch {
      /* app closing */
      return null;
    }
  }, []);

  // First paint: is there an index already?
  useEffect(() => {
    refreshState().then((s) => {
      setPhase(s && s.filesTotal > 0 ? "ready" : "empty");
    });
  }, [refreshState]);

  const pushToast = useCallback(
    (message: string, undoFileIds: string[] = [], undoPaths: string[] = []) => {
      const id = ++toastId.current;
      setToasts((ts) => [
        ...ts.slice(-3),
        { id, message, undoFileIds, undoPaths },
      ]);
    },
    [],
  );

  const onAction = useCallback(
    (message: string, undoFileIds: string[] = [], undoPaths: string[] = []) => {
      pushToast(message, undoFileIds, undoPaths);
      refreshState();
    },
    [pushToast, refreshState],
  );

  const dragDepth = useRef(0);
  useEffect(() => {
    const over = (e: DragEvent) => e.preventDefault();
    const enter = () => {
      dragDepth.current += 1;
      setDragging(true);
    };
    const leave = () => {
      dragDepth.current = Math.max(0, dragDepth.current - 1);
      if (dragDepth.current === 0) setDragging(false);
    };
    const drop = () => {
      dragDepth.current = 0;
      setDragging(false);
    };
    window.addEventListener("dragover", over);
    window.addEventListener("dragenter", enter);
    window.addEventListener("dragleave", leave);
    window.addEventListener("drop", drop);
    return () => {
      window.removeEventListener("dragover", over);
      window.removeEventListener("dragenter", enter);
      window.removeEventListener("dragleave", leave);
      window.removeEventListener("drop", drop);
    };
  }, []);

  const brand = (
    <header className="topbar">
      <span className="topbar__brand">
        <span className="topbar__eye" style={{ width: 24, display: "inline-flex" }}>
          <Eye className="eye" />
        </span>
        Draku
      </span>
      <span className="topbar__spacer" />
      <span className="topbar__status">
        <span className="dot" /> offline · nothing leaves this laptop
      </span>
    </header>
  );

  return (
    <div className="frame">
      {brand}
      {phase === "empty" && (
        <Empty
          dragging={dragging}
          onScanStarted={() => setPhase("scanning")}
        />
      )}
      {phase === "scanning" && (
        <Scanning progress={progress} onCancel={() => api.cancelScan()} />
      )}
      {phase === "ready" && state && <OneScreen state={state} onAction={onAction} />}
      <Toasts
        toasts={toasts}
        onExpire={(id) => setToasts((ts) => ts.filter((t) => t.id !== id))}
        onUndo={(t) => {
          setToasts((ts) => ts.map((x) => (x.id === t.id ? { ...t, id: t.id } : x)));
          refreshState();
        }}
      />
    </div>
  );
}
