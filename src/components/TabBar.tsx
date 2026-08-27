export type TabId =
  | "cognitive"
  | "screentime"
  | "trends"
  | "insights"
  | "journal"
  | "settings";

interface TabBarProps {
  active: TabId;
  onChange: (tab: TabId) => void;
}

const TABS: { id: TabId; label: string }[] = [
  { id: "cognitive", label: "Cognitive Map" },
  { id: "screentime", label: "Screen Time" },
  { id: "trends", label: "Trends" },
  { id: "insights", label: "Insights" },
  { id: "journal", label: "Journal" },
  { id: "settings", label: "Settings" },
];

export function TabBar({ active, onChange }: TabBarProps) {
  return (
    <nav className="flex flex-wrap gap-1 bg-[var(--surface-2)] p-1 rounded-xl border border-[var(--border)]">
      {TABS.map((t) => (
        <button
          key={t.id}
          type="button"
          onClick={() => onChange(t.id)}
          className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
            active === t.id
              ? "bg-cool-600 text-white shadow-sm"
              : "text-[var(--text-muted)] hover:text-[var(--text)] hover:bg-[var(--surface)]"
          }`}
        >
          {t.label}
        </button>
      ))}
    </nav>
  );
}
