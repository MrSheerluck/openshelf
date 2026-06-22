import type { Highlight, HighlightColor } from "./types";
import { api } from "$lib/api";

const STORAGE_PREFIX = "openshelf:highlights:";

export class HighlightsStore {
  bookId: string;
  highlights = $state<Highlight[]>([]);
  private loaded = false;

  constructor(bookId: string) {
    this.bookId = bookId;
  }

  async load(): Promise<void> {
    if (this.loaded) return;
    this.loaded = true;
    try {
      const res = await api(`/api/books/${this.bookId}/annotations`);
      if (res.ok) {
        const data = await res.json();
        this.highlights = data.map((a: any) => ({
          id: a.id,
          chapterIndex: a.chapter_index ?? 0,
          chapterLabel: null,
          cfiRange: a.cfi,
          text: a.text,
          color: a.color as HighlightColor,
          note: a.note,
          createdAt: new Date(a.created_at).getTime(),
        }));
        this.saveLocal();
        return;
      }
    } catch {}
    this.loadLocal();
  }

  private loadLocal(): void {
    try {
      const stored = localStorage.getItem(this.key());
      if (stored) this.highlights = JSON.parse(stored);
    } catch {}
  }

  private saveLocal(): void {
    try {
      localStorage.setItem(this.key(), JSON.stringify(this.highlights));
    } catch {}
  }

  async add(
    cfiRange: string,
    text: string,
    color: HighlightColor,
    chapterIndex: number,
    chapterLabel: string | null,
    note: string | null = null,
  ): Promise<Highlight> {
    const existing = this.highlights.find((h) => h.cfiRange === cfiRange);
    if (existing) {
      existing.color = color;
      existing.text = text;
      existing.chapterIndex = chapterIndex;
      existing.chapterLabel = chapterLabel;
      existing.note = note;
      this.saveLocal();
      this.syncUpdate(existing);
      return existing;
    }
    const tempId = crypto.randomUUID();
    const h: Highlight = {
      id: tempId,
      chapterIndex,
      chapterLabel,
      cfiRange,
      text,
      color,
      note,
      createdAt: Date.now(),
    };
    this.highlights.push(h);
    this.saveLocal();
    this.syncCreate(h);
    return h;
  }

  async remove(id: string): Promise<void> {
    this.highlights = this.highlights.filter((h) => h.id !== id);
    this.saveLocal();
    try {
      await api(`/api/books/${this.bookId}/annotations/${id}`, { method: "DELETE" });
    } catch {}
  }

  async removeByCfi(cfiRange: string): Promise<void> {
    const h = this.highlights.find((x) => x.cfiRange === cfiRange);
    if (h) await this.remove(h.id);
  }

  async updateNote(id: string, note: string | null): Promise<void> {
    const h = this.highlights.find((x) => x.id === id);
    if (!h) return;
    h.note = note;
    this.saveLocal();
    try {
      await api(`/api/books/${this.bookId}/annotations/${id}`, {
        method: "PUT",
        body: JSON.stringify({ note, color: h.color }),
      });
    } catch {}
  }

  async updateColor(id: string, color: HighlightColor): Promise<void> {
    const h = this.highlights.find((x) => x.id === id);
    if (!h) return;
    h.color = color;
    this.saveLocal();
    await this.syncUpdate(h);
  }

  setChapterLabel(id: string, chapterLabel: string | null): void {
    const h = this.highlights.find((x) => x.id === id);
    if (!h || h.chapterLabel === chapterLabel) return;
    h.chapterLabel = chapterLabel;
    this.saveLocal();
  }

  findByCfi(cfiRange: string): Highlight | undefined {
    return this.highlights.find((h) => h.cfiRange === cfiRange);
  }

  private async syncCreate(h: Highlight): Promise<void> {
    try {
      const res = await api(`/api/books/${this.bookId}/annotations`, {
        method: "POST",
        body: JSON.stringify({
          chapter_index: h.chapterIndex,
          cfi: h.cfiRange,
          text: h.text,
          note: h.note,
          color: h.color,
        }),
      });
      if (res.ok) {
        const created = await res.json();
        const idx = this.highlights.findIndex((x) => x.id === h.id);
        if (idx >= 0) {
          this.highlights[idx].id = created.id;
          this.saveLocal();
        }
      }
    } catch {}
  }

  private async syncUpdate(h: Highlight): Promise<void> {
    try {
      await api(`/api/books/${this.bookId}/annotations/${h.id}`, {
        method: "PUT",
        body: JSON.stringify({ note: h.note, color: h.color }),
      });
    } catch {}
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
