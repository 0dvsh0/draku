/** Quiet, per-file progress. "Reading your files. Nothing leaves this laptop." */
import type { ScanProgress } from "../lib/types";

export function Scanning({
  progress,
  onCancel,
}: {
  progress: ScanProgress | null;
  onCancel: () => void;
}) {
  const p = progress ?? { done: 0, total: 0, current: "starting…", stage: "reading" };
  const pct = p.total > 0 ? Math.round((p.done / p.total) * 100) : 0;
  const stage =
    p.stage === "model"
      ? "loading EmbeddingGemma from this laptop — no download"
      : p.stage === "done"
        ? "indexed"
        : "reading";

  return (
    <section className="scanning" aria-live="polite">
      <p className="label">{stage}</p>
      <div className="track" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
        <div className="track__fill" style={{ width: `${pct}%` }} />
      </div>
      <p className="scanning__file">{p.current || "\u00a0"}</p>
      <div className="scanning__meta">
        <span>{p.done}/{p.total} files</span>
        <button className="btn btn--ghost" onClick={onCancel}>cancel</button>
      </div>
      <p className="empty__note">Reading your files. Nothing leaves this laptop.</p>
    </section>
  );
}
