import { useMemo } from "react";
import { useTheme } from "../context/ThemeContext";

export function useChartTheme() {
  const { theme } = useTheme();

  return useMemo(() => {
    if (theme === "light") {
      return {
        grid: "#c5ced9",
        axis: "#6b788c",
        tooltipBg: "#e2e7ef",
        tooltipBorder: "#b4becd",
        tooltipText: "#3a4556",
      };
    }
    return {
      grid: "#1e293b",
      axis: "#64748b",
      tooltipBg: "#0f172a",
      tooltipBorder: "#334155",
      tooltipText: "#f1f5f9",
    };
  }, [theme]);
}
