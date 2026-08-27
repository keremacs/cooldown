import {
  Area,
  AreaChart,
  CartesianGrid,
  Legend,
  ReferenceArea,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { DashboardState } from "../types";
import { fatigueColor, zoneColor, zoneLabel } from "../lib/utils";
import { useChartTheme } from "../hooks/useChartTheme";

interface CognitiveMapTabProps {
  state: DashboardState;
}

export function CognitiveMapTab({ state }: CognitiveMapTabProps) {
  const chart = useChartTheme();
  const chartData = state.cognitive_history.map((b) => ({
    hour: b.hour,
    fatigue: Math.round(b.fatigue),
    zone: b.zone,
  }));

  // Pad with placeholder if no history yet.
  if (chartData.length === 0) {
    chartData.push({ hour: "Now", fatigue: Math.round(state.fatigue_score), zone: state.zone });
  }

  return (
    <div className="space-y-6">
      {/* Fatigue score hero */}
      <div className="flex items-center gap-6">
        <div
          className="relative flex h-28 w-28 shrink-0 items-center justify-center rounded-full border-4"
          style={{
            borderColor: fatigueColor(state.fatigue_score),
            boxShadow: `0 0 24px ${fatigueColor(state.fatigue_score)}33`,
          }}
        >
          <div className="text-center">
            <span
              className="text-3xl font-bold tabular-nums"
              style={{ color: fatigueColor(state.fatigue_score) }}
            >
              {Math.round(state.fatigue_score)}
            </span>
            <p className="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">
              Fatigue
            </p>
          </div>
        </div>

        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span
              className="inline-block h-2 w-2 rounded-full"
              style={{ backgroundColor: zoneColor(state.zone) }}
            />
            <span className="text-sm font-medium text-[var(--text)]">
              {zoneLabel(state.zone)} Zone
            </span>
          </div>
          <p className="text-[var(--text-muted)] text-sm leading-relaxed">
            {state.insight}
          </p>
        </div>
      </div>

      {/* Time-series chart with zone bands */}
      <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-4">
          Cognitive Load Today
        </h3>
        <ResponsiveContainer width="100%" height={260}>
          <AreaChart data={chartData} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
            {/* Zone reference bands */}
            <ReferenceArea y1={0} y2={40} fill="#22c55e" fillOpacity={0.06} />
            <ReferenceArea y1={40} y2={75} fill="#f59e0b" fillOpacity={0.06} />
            <ReferenceArea y1={75} y2={100} fill="#ef4444" fillOpacity={0.06} />

            <CartesianGrid strokeDasharray="3 3" stroke={chart.grid} />
            <XAxis
              dataKey="hour"
              tick={{ fill: chart.axis, fontSize: 11 }}
              axisLine={{ stroke: chart.axis }}
            />
            <YAxis
              domain={[0, 100]}
              tick={{ fill: chart.axis, fontSize: 11 }}
              axisLine={{ stroke: chart.axis }}
            />
            <Tooltip
              contentStyle={{
                background: chart.tooltipBg,
                border: `1px solid ${chart.tooltipBorder}`,
                borderRadius: 8,
                fontSize: 12,
                color: chart.tooltipText,
              }}
              formatter={(value: number) => [`${value}`, "Fatigue"]}
            />
            <Legend
              wrapperStyle={{ fontSize: 11, color: chart.axis }}
              payload={[
                { value: "Flow (0–40)", type: "square", color: "#22c55e" },
                { value: "Distraction (40–75)", type: "square", color: "#f59e0b" },
                { value: "Burnout (75+)", type: "square", color: "#ef4444" },
              ]}
            />
            <Area
              type="monotone"
              dataKey="fatigue"
              stroke="#6366f1"
              fill="url(#fatigueGradient)"
              strokeWidth={2}
              dot={{ r: 3, fill: "#6366f1" }}
            />
            <defs>
              <linearGradient id="fatigueGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#6366f1" stopOpacity={0.4} />
                <stop offset="100%" stopColor="#6366f1" stopOpacity={0.02} />
              </linearGradient>
            </defs>
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Quick stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <StatCard label="Deep Work" value={Math.round(state.deep_work_score)} suffix="/100" />
        <StatCard label="Context Switches" value={state.switches_last_30min} suffix="/30m" />
        <StatCard label="Error Events" value={state.errors_last_hour} suffix="/hr" />
        <StatCard
          label="Typing Cadence"
          value={Math.round(state.keystrokes_per_min)}
          suffix="cpm"
        />
      </div>
    </div>
  );
}

function StatCard({
  label,
  value,
  suffix,
}: {
  label: string;
  value: number;
  suffix: string;
}) {
  return (
    <div className="rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-4 py-3">
      <p className="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{label}</p>
      <p className="text-xl font-semibold tabular-nums text-[var(--text)] mt-0.5">
        {value}
        <span className="text-xs font-normal text-[var(--text-muted)] ml-1">{suffix}</span>
      </p>
    </div>
  );
}
