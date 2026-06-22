import type { ReaderSettings, ThemeName, Typography } from "./types";
import { defaultTypography } from "./themes";

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
        if (parsed.theme) this.theme = parsed.theme;
        if (parsed.typography) {
          this.typography = { ...defaultTypography, ...parsed.typography };
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
