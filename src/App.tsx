import { useEffect, useState } from "react";
import { TabBar, type TabId } from "./components/TabBar";
import { CognitiveMapTab } from "./components/CognitiveMapTab";
import { ScreenTimeTab } from "./components/ScreenTimeTab";
import { TrendsTab } from "./components/TrendsTab";
import { InsightsTab } from "./components/InsightsTab";
import { JournalTab } from "./components/JournalTab";
import { SettingsTab } from "./components/SettingsTab";
import { MiniWidget } from "./components/MiniWidget";
import { FatigueNotification, AlertHintToast } from "./components/FatigueNotification";
import { useCooldownState } from "./hooks/useCooldownState";
import { ThemeProvider, useTheme } from "./context/ThemeContext";

function isWidgetMode() {
  return new URLSearchParams(window.location.search).get("widget") === "1";
}

function Dashboard() {
  const [tab, setTab] = useState<TabId>("cognitive");
  const { theme, toggleTheme, setTheme } = useTheme();
  const {
    state,
    events,
    notification,
    hint,
    journal,
    dismissNotification,
    snoozeNotification,
    saveJournal,
    setFocusMode,
    setRetentionDays,
  } = useCooldownState();

  useEffect(() => {
    if (state.theme === "light" || state.theme === "dark") {
      setTheme(state.theme);
    }
  }, [state.theme, setTheme]);

  if (isWidgetMode()) {
    return <MiniWidget state={state} />;
  }

  return (
    <div className="h-screen flex flex-col bg-[var(--bg)] text-[var(--text)]">
      <header className="shrink-0 border-b border-[var(--border)] px-6 py-4 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">Cooldown</h1>
          <p className="text-xs text-[var(--text-muted)]">
            Cognitive load &amp; burnout tracker
            {state.focus_mode.active && (
              <span className="ml-2 text-cool-400">· Focus Mode ON</span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={toggleTheme}
            className="text-xs px-2 py-1 rounded border border-[var(--border)] text-[var(--text-muted)]"
            title="Toggle theme"
          >
            {theme === "dark" ? "☀" : "☾"}
          </button>
          <TabBar active={tab} onChange={setTab} />
        </div>
      </header>

      <main className="flex-1 overflow-y-auto px-6 py-6 max-w-3xl mx-auto w-full scroll-area">
        {tab === "cognitive" && <CognitiveMapTab state={state} />}
        {tab === "screentime" && <ScreenTimeTab state={state} />}
        {tab === "trends" && <TrendsTab state={state} />}
        {tab === "insights" && <InsightsTab state={state} events={events} />}
        {tab === "journal" && <JournalTab entries={journal} />}
        {tab === "settings" && (
          <SettingsTab
            state={state}
            onFocusMode={setFocusMode}
            onRetention={setRetentionDays}
          />
        )}
      </main>

      {notification && (
        <FatigueNotification
          notification={notification}
          onDismiss={dismissNotification}
          onSnooze={snoozeNotification}
          onSaveJournal={saveJournal}
        />
      )}
      {hint && <AlertHintToast message={hint.message} />}
    </div>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <Dashboard />
    </ThemeProvider>
  );
}
