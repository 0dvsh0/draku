/** The One Screen: ask anything + three cards + the footer truth. */
import { useEffect, useRef, useState } from "react";
import { api } from "../lib/api";
import type { ActionOk, AppState, SearchHit } from "../lib/types";
import { fmtBytes, relativeAge } from "../lib/human";

// ---------------------------------------------------------------------------

function AskBar({
  query,
  setQuery,
}: {
  query: string;
  setQuery: (q: string) => void;
}) {
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => {
    (document.getElementById("draku-ask") as HTMLInputElement | null)?.focus();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        ref.current?.focus();
        ref.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <>
      <label className="ask">
        <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" strokeWidth="2" />
          <path d="m20 20-4-4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </svg>
        <input
          id="draku-ask"
          ref={ref}
          type="search"
          placeholder="Ask anything."
          autoComplete="off"
          value={query}
          aria-label="Ask anything about your files"
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              setQuery("");
              ref.current?.blur();
            }
          }}
        />
        <kbd>Ctrl K</kbd>
      </label>
      <div className="hints">
        {["Wi-Fi password", "blood test", "laptop warranty", "boarding pass"].map((h) => (
          <button key={h} onClick={() => setQuery(h)}>
            {h}
          </button>
        ))}
      </div>
    </>
  );
}

// ---------------------------------------------------------------------------

function Results({ hits, query }: { hits: SearchHit[]; query: string }) {
  if (!query.trim()) return null;
  return (
    <div className="results" aria-live="polite">
      {hits.length === 0 ? (
        <p className="empty-results">nothing in there reads like “{query}” — yet</p>
      ) : (
        hits.map((h) => (
          <article className="hit" key={h.fileId}>
            <span className="hit__score">
              <b>{Math.round(h.score * 100)}%</b>
              <i>SIMILAR</i>
            </span>
            <div className="hit__body">
              <p className="hit__name">{h.name}</p>
              <p className="hit__meta">
                {h.category} · {h.path.split(/[\\/]/).pop()}
              </p>
            </div>
            <button className="btn btn--ghost" onClick={() => api.reveal(h.path)}>
              Show
            </button>
          </article>
        ))
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------

function DeadlinesCard({
  state,
  onCalendar,
}: {
  state: AppState;
  onCalendar: (fileId: string) => void;
}) {
  const [confirming, setConfirming] = useState<string | null>(null);
  const next = state.deadlines[0];
  return (
    <article className="card card--date">
      <header className="card__head">
        <span className="card__num">{state.deadlines.length}</span>
        <span className="card__what">{state.deadlines.length === 1 ? "deadline" : "deadlines"}</span>
      </header>
      {next ? (
        <>
          <ul className="card__list">
            {state.deadlines.slice(0, 4).map((d) => (
              <li key={d.fileId}>
                <span title={d.name}>{d.label || d.name}</span>
                <b>
                  {d.daysLeft === 0
                    ? "today"
                    : d.daysLeft === 1
                      ? "tomorrow"
                      : `in ${d.daysLeft} days`}
                </b>
              </li>
            ))}
          </ul>
          <button
            className={"btn btn--blood card__btn" + (confirming === next.fileId ? " is-confirm" : "")}
            onClick={() => {
              if (confirming === next.fileId) {
                setConfirming(null);
                onCalendar(next.fileId);
              } else {
                setConfirming(next.fileId);
              }
            }}
            onBlur={() => setConfirming(null)}
          >
            {confirming === next.fileId ? "Tap again to set it" : "Add to calendar"}
          </button>
        </>
      ) : (
        <p className="card__empty">No dates are breathing down your neck.</p>
      )}
    </article>
  );
}

function SecretCard({
  state,
  onSafeCopy,
  onReveal,
}: {
  state: AppState;
  onSafeCopy: (fileId: string) => void;
  onReveal: (path: string) => void;
}) {
  const [confirming, setConfirming] = useState(false);
  const found = state.secrets[0];
  return (
    <article className="card card--secret">
      <header className="card__head">
        <span className="card__num">{state.secrets.length}</span>
        <span className="card__what">
          exposed {state.secrets.length === 1 ? "secret" : "secrets"}
        </span>
      </header>
      {found ? (
        <>
          <div className="card__secret">
            <small>{found.kind.replace("_", " ").toUpperCase()}</small>
            {found.preview || "in the metadata"}
          </div>
          <p className="card__footnote">
            found in <em onClick={() => onReveal(found.path)} style={{ cursor: "pointer" }}>{found.name}</em>
            {found.hasGps ? " — with a hidden location tag" : ""}
          </p>
          <button
            className={"btn btn--blood card__btn" + (confirming ? " is-confirm" : "")}
            onClick={() => {
              if (confirming) {
                setConfirming(false);
                onSafeCopy(found.fileId);
              } else {
                setConfirming(true);
              }
            }}
            onBlur={() => setConfirming(false)}
          >
            {confirming ? "Tap again to copy" : "Make a safe copy"}
          </button>
        </>
      ) : (
        <p className="card__empty">Nothing is lying around open.</p>
      )}
    </article>
  );
}

function LetGoCard({
  state,
  onSetAside,
  onSetAsidesAll,
}: {
  state: AppState;
  onSetAside: (fileId: string) => void;
  onSetAsidesAll: () => void;
}) {
  const [confirming, setConfirming] = useState(false);
  const items = state.rest;
  return (
    <article className="card card--letgo">
      <header className="card__head">
        <span className="card__num">{items.length}</span>
        <span className="card__what">files ready to let go</span>
      </header>
      {items.length > 0 ? (
        <>
          <ul className="card__list">
            {items.slice(0, 4).map((r) => (
              <li key={r.fileId}>
                <span title={r.name}>{r.name}</span>
                <b className="muted" style={{ color: "var(--ash-2)" }}>{fmtBytes(r.size)}</b>
              </li>
            ))}
          </ul>
          <p className="card__footnote">
            {fmtBytes(state.restFreed)} would move to Rest, never deleted ·{" "}
            {relativeAge(items[0])}
          </p>
          <button
            className={"btn btn--blood card__btn" + (confirming ? " is-confirm" : "")}
            onClick={() => {
              if (confirming) {
                setConfirming(false);
                onSetAsidesAll();
              } else {
                setConfirming(true);
              }
            }}
            onBlur={() => setConfirming(false)}
          >
            {confirming ? `Tap again — set aside ${items.length}` : `Set aside (${fmtBytes(state.restFreed)})`}
          </button>
          <button className="btn btn--ghost card__btn" onClick={() => onSetAside(items[0].fileId)}>
            just the first one
          </button>
        </>
      ) : (
        <p className="card__empty">Everything here has life in it.</p>
      )}
    </article>
  );
}

// ---------------------------------------------------------------------------

export function OneScreen({
  state,
  onAction,
}: {
  state: AppState;
  onAction: (msg: string, undoFileIds?: string[], undoPaths?: string[]) => void;
}) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [searching, setSearching] = useState(false);

  // Debounced real search on the local index.
  useEffect(() => {
    if (!query.trim()) {
      setHits([]);
      return;
    }
    setSearching(true);
    const t = setTimeout(async () => {
      try {
        setHits(await api.search(query));
      } finally {
        setSearching(false);
      }
    }, 180);
    return () => {
      clearTimeout(t);
      setSearching(false);
    };
  }, [query]);

  const run = async (fn: () => Promise<ActionOk>) => {
    try {
      const r = await fn();
      // Any real action re-tints the cards via the App's refresh on toast.
      onAction(r.message, r.undoFileIds, r.undoPaths);
    } catch (err) {
      onAction(String(err));
    }
  };

  return (
    <section className="screen" aria-label="One screen">
      <div className="askrow">
        <AskBar query={query} setQuery={setQuery} />
      </div>
      <Results hits={hits} query={searching ? "" : query} />

      <div className="cards">
        <DeadlinesCard state={state} onCalendar={(id) => run(() => api.addToCalendar(id))} />
        <SecretCard
          state={state}
          onSafeCopy={(id) => run(() => api.makeSafeCopy(id))}
          onReveal={api.reveal}
        />
        <LetGoCard
          state={state}
          onSetAside={(id) => run(() => api.setAside(id))}
          onSetAsidesAll={() =>
            run(() => api.setAsideMany(state.rest.map((r) => r.fileId)))
          }
        />
      </div>

      <div className="truth">
        <p>
          <em>Nothing changes on your device until you press a button.</em>
        </p>
        <span className="topbar__status">
          <span className="dot" /> {state.filesTotal} files indexed · airplane safe
        </span>
      </div>

      <svg className="wing-edge" viewBox="0 0 1440 26" preserveAspectRatio="none" aria-hidden="true">
        <path d="M0 26 V8 Q60 26 120 8 Q180 26 240 8 Q300 26 360 8 Q420 26 480 8 Q540 26 600 8 Q660 26 720 8 Q780 26 840 8 Q900 26 960 8 Q1020 26 1080 8 Q1140 26 1200 8 Q1260 26 1320 8 Q1380 26 1440 8 V26 Z" />
      </svg>
    </section>
  );
}
