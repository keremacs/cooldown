import type { JournalEntry } from "../types";

export function JournalTab({ entries }: { entries: JournalEntry[] }) {
  if (entries.length === 0) {
    return (
      <p className="text-sm text-[var(--text-muted)] text-center py-12">
        No journal entries yet. They appear when you take a break and log what you were working on.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      {entries.map((e) => (
        <article
          key={e.id}
          className="rounded-lg border border-[var(--border)] bg-[var(--surface)] px-4 py-3"
        >
          <time className="text-[10px] uppercase text-[var(--text-muted)]">
            {new Date(e.ts * 1000).toLocaleString()}
          </time>
          <p className="text-sm mt-1 text-[var(--text)]">{e.text}</p>
        </article>
      ))}
    </div>
  );
}
