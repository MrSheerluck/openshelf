import type { FontFamily, Theme, Typography } from "./types";

export const themes: Theme[] = [
  { name: "light", bg: "#ffffff", fg: "#1a1a1a", label: "Light" },
  { name: "sepia", bg: "#f4ecd8", fg: "#3a2f1c", label: "Sepia" },
  { name: "dark", bg: "#1a1a1a", fg: "#d4d4d4", label: "Dark" },
];

export const fontFamilies: { value: FontFamily; label: string; css: string }[] = [
  { value: "serif", label: "Serif", css: 'Georgia, "Iowan Old Style", "Palatino Linotype", "Times New Roman", serif' },
  { value: "sans", label: "Sans", css: 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif' },
  { value: "mono", label: "Mono", css: 'ui-monospace, "SF Mono", Menlo, Consolas, monospace' },
];

export function fontFamilyCss(family: FontFamily): string {
  return fontFamilies.find((f) => f.value === family)?.css ?? fontFamilies[0].css;
}

export const defaultTypography: Typography = {
  fontFamily: "serif",
  fontSize: 100,
  lineHeight: 1.6,
  margin: 24,
  align: "left",
};
