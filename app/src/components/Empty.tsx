/** Empty state = a drop zone and a native folder picker. Nothing else. */
import { useState } from "react";
import { api } from "../lib/api";
import { Eyes } from "./Eye";

export function Empty({
  onScanStarted,
  dragging,
}: {
  onScanStarted: () => void;
  dragging: boolean;
}) {
  const [busy, setBusy] = useState(false);

  const pick = async () => {
    setBusy(true);
    try {
      const dir = await api.openFolderDialog();
      if (dir) {
        onScanStarted();
        await api.startScan([dir]);
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="empty">
      <div className={"empty__drop" + (dragging ? " is-over" : "")}>
        <Eyes />
        <h1 className="empty__title">
          Give me a folder.<br />
          <em>I'll read it, right here.</em>
        </h1>
        <p className="empty__lede">
          Drop screenshots, photos or PDFs — or point me at Downloads. I'll name
          what's cryptic, catch the dates that matter, guard what's private and
          set aside what's dead.
        </p>
        <div className="empty__actions">
          <button className="btn btn--blood" onClick={pick} disabled={busy}>
            <svg width="17" height="17" viewBox="0 0 24 24" aria-hidden="true">
              <path
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M3 7h6l2 2h10v10H3zM3 7V5h8l2 2"
              />
            </svg>
            Choose a folder…
          </button>
        </div>
        <p className="empty__note">or drop files anywhere on me</p>
      </div>
      <p className="empty__note">
        EmbeddingGemma ships inside this app — the very first scan needs no
        download, and never opens the network
      </p>
      <p className="empty__note">
        reads everything on this laptop · never opens the network · v0.1
      </p>
    </section>
  );
}
