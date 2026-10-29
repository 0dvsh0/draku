/** The mascot's eyes — round white rings, crimson iris, black flames. */
export function Eye({ className = "", delay = 0 }: { className?: string; delay?: number }) {
  return (
    <svg viewBox="0 0 100 100" className={className} aria-hidden="true">
      <circle cx="50" cy="50" r="48" fill="#080708" />
      <circle cx="50" cy="50" r="44" fill="#EAE6E6" stroke="#080708" strokeWidth="2.5" />
      <circle cx="50" cy="50" r="39.5" fill="none" stroke="#080708" strokeWidth="1.2" opacity=".55" />
      <g>
        <circle cx="50" cy="50" r="34" fill="#080708" />
        <circle cx="50" cy="50" r="32" fill="#7A1D25" />
        <g clipPath="circle(33px at 50px 50px)">
          <path
            className="eye__flame"
            style={{ animationDelay: `${-delay}s` }}
            d="M18 86 C16 72 24 66 22 54 C28 60 31 66 33 70 C31 56 36 46 41 34 C44 46 50 52 49 64 C53 56 56 50 55 40 C63 50 67 58 65 70 C68 64 71 60 72 52 C80 62 84 74 82 86 Z"
            fill="#080708"
            transform="translate(9 9) scale(.82)"
          />
        </g>
        <ellipse cx="38" cy="34" rx="5" ry="3" fill="#EAE6E6" opacity=".35" transform="rotate(-30 38 34)" />
      </g>
    </svg>
  );
}

export function Eyes({ size = 66 }: { size?: number }) {
  return (
    <span className="empty__eyes" style={{ fontSize: size }}>
      <Eye className="eye" />
      <Eye className="eye" delay={1.1} />
    </span>
  );
}

/** Tiny inline blue flame for confirmed interrupts of the eye. */
export function SparkIcon() {
  return (
    <svg viewBox="0 0 24 24" width="15" height="15" aria-hidden="true">
      <path
        fill="currentColor"
        d="M12 3l1.9 4.7L18.5 10l-4.6 2.2L12 17l-1.9-4.8L5.5 10l4.6-2.3L12 3Zm6.8 9.6l.9 2.3 2.3.9-2.3.9-.9 2.3-.9-2.3-2.3-.9 2.3-.9.9-2.3Z"
      />
    </svg>
  );
}
