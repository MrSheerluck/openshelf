import type { Highlight, HighlightColor } from "./types";

const STORAGE_PREFIX = "openshelf:highlights:";

export class HighlightsStore {
  bookId: string;
  highlights = $state<Highlight[]>([]);

  constructor(bookId: string) {
    this.bookId = bookId;
  }

  load(): void {
    try {
      const stored = localStorage.getItem(this.key());
      if (stored) {
        this.highlights = JSON.parse(stored);
      }
    } catch {}
  }

  save(): void {
    try {
      localStorage.setItem(this.key(), JSON.stringify(this.highlights));
    } catch {}
  }

  add(cfiRange: string, text: string, color: HighlightColor, note: string | null = null): Highlight {
    const existing = this.highlights.find((h) => h.cfiRange === cfiRange);
    if (existing) {
      existing.color = color;
      existing.text = text;
      existing.note = note;
      this.save();
      return existing;
    }
    const h: Highlight = {
      id: crypto.randomUUID(),
      cfiRange,
      text,
      color,
      note,
      createdAt: Date.now(),
    };
    this.highlights.push(h);
    this.save();
    return h;
  }

  remove(id: string): void {
    this.highlights = this.highlights.filter((h) => h.id !== id);
    this.save();
  }

  removeByCfi(cfiRange: string): void {
    this.highlights = this.highlights.filter((h) => h.cfiRange !== cfiRange);
    this.save();
  }

  updateNote(id: string, note: string | null): void {
    const h = this.highlights.find((x) => x.id === id);
    if (h) {
      h.note = note;
      this.save();
    }
  }

  findByCfi(cfiRange: string): Highlight | undefined {
    return this.highlights.find((h) => h.cfiRange === cfiRange);
  }

  private key(): string {
    return `${STORAGE_PREFIX}${this.bookId}`;
  }
}

export const highlightColors: { value: HighlightColor; label: string; css: string }[] = [
  { value: "yellow", label: "Yellow", css: "rgba(255, 213, 79, 0.35)" },
  { value: "green",  label: "Green",  css: "rgba(102, 187, 106, 0.35)" },
  { value: "blue",   label: "Blue",   css: "rgba(66, 165, 245, 0.35)" },
  { value: "pink",   label: "Pink",   css: "rgba(240, 98, 146, 0.35)" },
];

export function highlightColorCss(color: HighlightColor): string {
  return highlightColors.find((c) => c.value === color)?.css ?? highlightColors[0].css;
}
