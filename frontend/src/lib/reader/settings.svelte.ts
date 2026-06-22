import type { ReaderSettings, ThemeName, Typography } from "./types";
import { defaultTypography, fontFamilies, themes } from "./themes";

const validFonts = new Set(fontFamilies.map((f) => f.value));
const validThemes = new Set(themes.map((t) => t.name));

export class ReaderSettingsStore {
  bookId: string;
  theme = $state<ThemeName>("light");
  typography = $state<Typography>({ ...defaultTypography });

  constructor(bookId: string) {
    this.bookId = bookId;
  }

  load(): { cfi?: string } {
    try {
      const stored = localStorage.getItem(this.key());
      if (stored) {
        const parsed = JSON.parse(stored) as Partial<ReaderSettings>;
        if (parsed.theme && validThemes.has(parsed.theme)) this.theme = parsed.theme;
        if (parsed.typography) {
          const merged = { ...defaultTypography, ...parsed.typography };
          if (!validFonts.has(merged.fontFamily)) merged.fontFamily = defaultTypography.fontFamily;
          this.typography = merged;
        }
        return { cfi: parsed.cfi };
      }
    } catch {}
    return {};
  }

  save(extra?: { cfi?: string }): void {
    try {
      const payload: ReaderSettings = {
        theme: this.theme,
        typography: { ...this.typography },
        cfi: extra?.cfi,
      };
      localStorage.setItem(this.key(), JSON.stringify(payload));
    } catch {}
  }

  private key(): string {
    return `openshelf:reader:${this.bookId}`;
  }
}
