import { useState } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { DashboardState } from "../types";
import { useChartTheme } from "../hooks/useChartTheme";

export function TrendsTab({ state }: { state: DashboardState }) {
  const chart = useChartTheme();
  const [period, setPeriod] = useState<"weekly" | "monthly">("weekly");
  const data = period === "weekly" ? state.weekly_trend : state.monthly_trend;

  return (
    <div className="space-y-6">
      <div className="flex gap-2">
        {(["weekly", "monthly"] as const).map((p) => (
          <button
            key={p}
            type="button"
            onClick={() => setPeriod(p)}
            className={`px-3 py-1.5 text-xs rounded-lg border ${
              period === p
                ? "border-cool-500 bg-cool-600/20 text-cool-600 dark:text-cool-400"
                : "border-[var(--border)] text-[var(--text-muted)]"
            }`}
          >
            {p === "weekly" ? "Weekly" : "Monthly"}
          </button>
        ))}
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-4">
          Fatigue Trend
        </h3>
        {data.length === 0 ? (
          <p className="text-sm text-[var(--text-muted)]">Not enough data yet — keep Cooldown running.</p>
        ) : (
          <ResponsiveContainer width="100%" height={220}>
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" stroke={chart.grid} />
              <XAxis dataKey="label" tick={{ fill: chart.axis, fontSize: 10 }} />
              <YAxis domain={[0, 100]} tick={{ fill: chart.axis, fontSize: 10 }} />
              <Tooltip contentStyle={{
                background: chart.tooltipBg,
                border: `1px solid ${chart.tooltipBorder}`,
                color: chart.tooltipText,
              }} />
              <Line type="monotone" dataKey="avg_fatigue" stroke="#6366f1" strokeWidth={2} dot={false} name="Fatigue" />
              <Line type="monotone" dataKey="avg_deep_work" stroke="#22c55e" strokeWidth={2} dot={false} name="Deep Work" />
            </LineChart>
          </ResponsiveContainer>
        )}
      </div>

      {data.length > 0 && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-4">
            Context Switches
          </h3>
          <ResponsiveContainer width="100%" height={160}>
            <BarChart data={data}>
              <XAxis dataKey="label" tick={{ fill: chart.axis, fontSize: 10 }} />
              <YAxis tick={{ fill: chart.axis, fontSize: 10 }} />
              <Tooltip contentStyle={{
                background: chart.tooltipBg,
                border: `1px solid ${chart.tooltipBorder}`,
                color: chart.tooltipText,
              }} />
              <Bar dataKey="avg_switches" fill="#f59e0b" name="Switches" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}
