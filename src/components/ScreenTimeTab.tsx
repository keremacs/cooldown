import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import type { DashboardState } from "../types";
import { formatDuration, formatPercent } from "../lib/utils";
import { useChartTheme } from "../hooks/useChartTheme";

interface ScreenTimeTabProps {
  state: DashboardState;
}

const CATEGORIES = [
  { key: "ide_secs" as const, label: "IDE", color: "#6366f1" },
  { key: "browser_secs" as const, label: "Browser", color: "#22c55e" },
  { key: "communication_secs" as const, label: "Communication", color: "#f59e0b" },
  { key: "other_secs" as const, label: "Other", color: "#64748b" },
];

export function ScreenTimeTab({ state }: ScreenTimeTabProps) {
  const chart = useChartTheme();
  const { screen_time: st } = state;
  const total =
    st.ide_secs + st.browser_secs + st.communication_secs + st.other_secs;

  const chartData = CATEGORIES.map((c) => ({
    name: c.label,
    value: st[c.key],
    color: c.color,
  })).filter((d) => d.value > 0);

  if (chartData.length === 0) {
    chartData.push({ name: "Collecting…", value: 1, color: "#334155" });
  }

  return (
    <div className="space-y-6">
      <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-4">
          App Usage Breakdown
        </h3>

        <div className="flex flex-col md:flex-row items-center gap-6">
          <ResponsiveContainer width="100%" height={220}>
            <PieChart>
              <Pie
                data={chartData}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={90}
                paddingAngle={3}
                dataKey="value"
              >
                {chartData.map((entry) => (
                  <Cell key={entry.name} fill={entry.color} stroke="transparent" />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{
                  background: chart.tooltipBg,
                  border: `1px solid ${chart.tooltipBorder}`,
                  borderRadius: 8,
                  fontSize: 12,
                  color: chart.tooltipText,
                }}
                formatter={(value: number) => formatDuration(value)}
              />
            </PieChart>
          </ResponsiveContainer>

          <div className="flex-1 w-full space-y-3">
            {CATEGORIES.map((c) => {
              const secs = st[c.key];
              return (
                <div key={c.key} className="flex items-center gap-3">
                  <span
                    className="h-2.5 w-2.5 rounded-full shrink-0"
                    style={{ backgroundColor: c.color }}
                  />
                  <span className="text-sm text-[var(--text)] flex-1">{c.label}</span>
                  <span className="text-sm tabular-nums text-[var(--text-muted)]">
                    {formatDuration(secs)}
                  </span>
                  <span className="text-xs tabular-nums text-[var(--text-muted)] w-10 text-right opacity-70">
                    {formatPercent(secs, total)}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* Active window */}
      <div className="rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-4 py-3">
        <p className="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">
          Active Window
        </p>
        <p className="text-sm text-[var(--text)] mt-1 truncate">
          {state.active_window || "—"}
        </p>
      </div>

      <p className="text-xs text-[var(--text-muted)] text-center opacity-80">
        Screen time is tracked locally from active window titles — no cloud sync.
      </p>
    </div>
  );
}
