import type { FontFamily, Theme, ThemeName, Typography } from "./types";

export const themes: Theme[] = [
  { name: "light",   bg: "#ffffff", fg: "#1a1a1a", label: "White",  isDark: false },
  { name: "cream",   bg: "#f7f3e9", fg: "#3a3226", label: "Cream",  isDark: false },
  { name: "sepia",   bg: "#f4ecd8", fg: "#3a2f1c", label: "Sepia",  isDark: false },
  { name: "green",   bg: "#e4ede4", fg: "#2d3a2d", label: "Mint",   isDark: false },
  { name: "dark",    bg: "#1a1a1a", fg: "#d4d4d4", label: "Dark",   isDark: true },
  { name: "night",   bg: "#0a0a0a", fg: "#c0c0c0", label: "Night",  isDark: true },
];

export function themeByName(name: ThemeName): Theme {
  return themes.find((t) => t.name === name) ?? themes[0];
}

export const fontFamilies: { value: FontFamily; label: string; css: string }[] = [
  { value: "literata",   label: "Literata",   css: '"Literata", Georgia, serif' },
  { value: "andika",     label: "Andika",     css: '"Andika", sans-serif' },
  { value: "shantell",   label: "Shantell",   css: '"Shantell Sans", sans-serif' },
  { value: "noto",       label: "Noto Sans",  css: '"Noto Sans", sans-serif' },
  { value: "libertinus", label: "Libertinus", css: '"Libertinus Mono", monospace' },
];

export function fontFamilyCss(family: FontFamily): string {
  return fontFamilies.find((f) => f.value === family)?.css ?? fontFamilies[0].css;
}

export const defaultTypography: Typography = {
  fontFamily: "literata",
  fontSize: 120,
  lineHeight: 1.5,
  margin: 20,
  align: "left",
};
