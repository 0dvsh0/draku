/** Toasts with real Undo — the restore is one tap, no dialogs. */
import { useEffect } from "react";
import { api } from "../lib/api";

export interface Toast {
  id: number;
  message: string;
  undoFileIds: string[];
  undoPaths: string[];
}

export function Toasts({
  toasts,
  onExpire,
  onUndo,
}: {
  toasts: Toast[];
  onExpire: (id: number) => void;
  onUndo: (t: Toast) => void;
}) {
  return (
    <div className="toasts" aria-live="polite">
      {toasts.map((t) => (
        <ToastRow key={t.id} toast={t} onExpire={onExpire} onUndo={onUndo} />
      ))}
    </div>
  );
}

function ToastRow({
  toast,
  onExpire,
  onUndo,
}: {
  toast: Toast;
  onExpire: (id: number) => void;
  onUndo: (t: Toast) => void;
}) {
  useEffect(() => {
    const t = setTimeout(() => onExpire(toast.id), 7000);
    return () => clearTimeout(t);
  }, [toast.id, onExpire]);

  const hasUndo = toast.undoFileIds.length > 0 || toast.undoPaths.length > 0;

  return (
    <div className="toast">
      <p className="toast__msg">{toast.message}</p>
      {hasUndo && (
        <button
          className="btn btn--ghost"
          onClick={async () => {
            let message = "Undone.";
            try {
              if (toast.undoFileIds.length > 0) {
                const r = await api.restoreMany(toast.undoFileIds);
                message = r.message;
              }
            } catch (err) {
              message = String(err);
            }
            try {
              if (toast.undoPaths.length > 0) {
                await api.undoGenerated(toast.undoPaths);
              }
            } catch (err) {
              message = String(err);
            }
            onUndo({ ...toast, message, undoFileIds: [], undoPaths: [] });
            onExpire(toast.id);
          }}
        >
          Undo
        </button>
      )}
      <button className="btn btn--ghost" onClick={() => onExpire(toast.id)} aria-label="Dismiss">
        ✕
      </button>
    </div>
  );
}
