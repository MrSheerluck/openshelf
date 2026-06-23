import type { ReaderSettings, ThemeName, Typography } from "./types";
import { defaultTypography, fontFamilies, themes } from "./themes";
import { api } from "$lib/api";

const validFonts = new Set(fontFamilies.map((f) => f.value));
const validThemes = new Set(themes.map((t) => t.name));

export class ReaderSettingsStore {
  bookId: string;
  theme = $state<ThemeName>("light");
  typography = $state<Typography>({ ...defaultTypography });
  private loaded = false;

  constructor(bookId: string) {
    this.bookId = bookId;
  }

  async load(initialCfi?: string | null): Promise<void> {
    if (this.loaded) return;
    this.loaded = true;
    const local = this.loadLocal();
    if (local.cfi && !initialCfi) this.applySettings(local);
    try {
      const res = await api(`/api/books/${this.bookId}/settings`);
      if (res.ok) {
        const data = await res.json();
        if (data) {
          this.theme = validThemes.has(data.theme) ? data.theme : this.theme;
          const fam = data.font_family as Typography["fontFamily"];
          this.typography = {
            ...defaultTypography,
            fontFamily: validFonts.has(fam)
              ? fam
              : defaultTypography.fontFamily,
            fontSize: data.font_size ?? defaultTypography.fontSize,
          };
          this.saveLocal();
        }
      }
    } catch {}
    if (initialCfi) {
      this.saveLocal(initialCfi);
    }
  }

  save(extra?: { cfi?: string }): void {
    this.saveLocal(extra?.cfi);
    this.syncToBackend();
  }

  clearCfi(): void {
    this.saveLocal(undefined);
  }

  private applySettings(s: {
    theme?: ThemeName;
    typography?: Partial<Typography>;
  }): void {
    if (s.theme && validThemes.has(s.theme)) this.theme = s.theme;
    if (s.typography) {
      const merged = { ...defaultTypography, ...s.typography };
      if (!validFonts.has(merged.fontFamily))
        merged.fontFamily = defaultTypography.fontFamily;
      this.typography = merged;
    }
  }

  private loadLocal(): {
    cfi?: string;
    theme?: ThemeName;
    typography?: Partial<Typography>;
  } {
    try {
      const stored = localStorage.getItem(this.key());
      if (stored) {
        const parsed = JSON.parse(stored) as Partial<ReaderSettings>;
        return {
          cfi: parsed.cfi,
          theme: parsed.theme,
          typography: parsed.typography,
        };
      }
    } catch {}
    return {};
  }

  private saveLocal(cfi?: string): void {
    try {
      const payload: ReaderSettings = {
        theme: this.theme,
        typography: { ...this.typography },
        cfi,
      };
      localStorage.setItem(this.key(), JSON.stringify(payload));
    } catch {}
  }

  private syncTimeout: ReturnType<typeof setTimeout> | null = null;

  private syncToBackend(): void {
    if (this.syncTimeout) clearTimeout(this.syncTimeout);
    this.syncTimeout = setTimeout(async () => {
      try {
        await api(`/api/books/${this.bookId}/settings`, {
          method: "PUT",
          body: JSON.stringify({
            theme: this.theme,
            font_family: this.typography.fontFamily,
            font_size: this.typography.fontSize,
          }),
        });
      } catch {}
    }, 1000);
  }

  private key(): string {
    return `openshelf:reader:${this.bookId}`;
  }
}
