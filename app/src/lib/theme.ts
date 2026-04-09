import { writable } from "svelte/store";

export type Theme = "auto" | "light" | "dark";

export const theme = writable<Theme>("auto");

export function applyTheme(t: Theme): void {
  const isDark =
    t === "dark" ||
    (t === "auto" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.classList.toggle("dark", isDark);
}

export function cycleTheme(): void {
  theme.update((current) => {
    const next: Theme =
      current === "auto" ? "light" : current === "light" ? "dark" : "auto";
    localStorage.setItem("theme", next);
    applyTheme(next);
    return next;
  });
}

export const themeIcons: Record<Theme, string> = {
  auto: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z",
  light:
    "M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41M12 7a5 5 0 1 0 0 10A5 5 0 0 0 12 7Z",
  dark: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79Z",
};
