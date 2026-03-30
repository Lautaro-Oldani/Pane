import { useEffect, useState } from "react";

export type Theme = "dark" | "light" | "system";

export function useTheme(themeSetting: string) {
  const [resolved, setResolved] = useState<"dark" | "light">("dark");

  useEffect(() => {
    if (themeSetting === "light") {
      setResolved("light");
      return;
    }
    if (themeSetting === "dark") {
      setResolved("dark");
      return;
    }

    // "system" — detectar preferencia de Windows
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    setResolved(mq.matches ? "dark" : "light");

    const handler = (e: MediaQueryListEvent) => setResolved(e.matches ? "dark" : "light");
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [themeSetting]);

  // Aplicar clase al document
  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolved === "dark");
    document.documentElement.classList.toggle("light", resolved === "light");
  }, [resolved]);

  return resolved;
}
