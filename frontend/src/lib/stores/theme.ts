import { writable } from "svelte/store";
import { browser } from "$app/environment";

type Theme = "light" | "dark";

const getInitialTheme = (): Theme => {
  if (browser) {
    const stored = localStorage.getItem("theme");
    if (stored === "light" || stored === "dark") return stored;
    if (window.matchMedia("(prefers-color-scheme: dark)").matches)
      return "dark";
  }
  return "light";
};

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>(getInitialTheme());

  return {
    subscribe,
    toggle: () =>
      update((current) => {
        const next = current === "light" ? "dark" : "light";
        if (browser) {
          localStorage.setItem("theme", next);
          document.documentElement.classList.toggle("dark", next === "dark");
        }
        return next;
      }),
    set: (theme: Theme) => {
      if (browser) {
        localStorage.setItem("theme", theme);
        document.documentElement.classList.toggle("dark", theme === "dark");
      }
      set(theme);
    },
  };
}

export const theme = createThemeStore();
