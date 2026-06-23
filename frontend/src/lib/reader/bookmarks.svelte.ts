import { api } from "$lib/api";
import type { Bookmark } from "./types";

export class BookmarksStore {
  bookId: string;
  bookmarks = $state<Bookmark[]>([]);
  private loaded = false;

  constructor(bookId: string) {
    this.bookId = bookId;
  }

  async load(): Promise<void> {
    if (this.loaded) return;
    this.loaded = true;
    try {
      const res = await api(`/api/books/${this.bookId}/bookmarks`);
      if (!res.ok) return;
      const data = await res.json();
      this.bookmarks = data.map((bookmark: any) => ({
        id: bookmark.id,
        chapterIndex: bookmark.chapter_index ?? 0,
        chapterLabel: bookmark.label ?? null,
        cfi: bookmark.cfi,
        createdAt: new Date(bookmark.created_at).getTime(),
      }));
    } catch {}
  }

  findByCfi(cfi: string): Bookmark | undefined {
    return this.bookmarks.find((bookmark) => bookmark.cfi === cfi);
  }

  async add(
    cfi: string,
    chapterIndex: number,
    chapterLabel: string | null,
  ): Promise<Bookmark | null> {
    try {
      const res = await api(`/api/books/${this.bookId}/bookmarks`, {
        method: "POST",
        body: JSON.stringify({
          cfi,
          chapter_index: chapterIndex,
          label: chapterLabel,
        }),
      });
      if (!res.ok) return null;
      const bookmark = await res.json();
      const existing = this.findByCfi(bookmark.cfi);
      if (existing) return existing;
      const next: Bookmark = {
        id: bookmark.id,
        chapterIndex: bookmark.chapter_index ?? 0,
        chapterLabel: bookmark.label ?? null,
        cfi: bookmark.cfi,
        createdAt: new Date(bookmark.created_at).getTime(),
      };
      this.bookmarks = [...this.bookmarks, next];
      return next;
    } catch {
      return null;
    }
  }

  async remove(id: string): Promise<void> {
    this.bookmarks = this.bookmarks.filter((bookmark) => bookmark.id !== id);
    try {
      await api(`/api/books/${this.bookId}/bookmarks/${id}`, {
        method: "DELETE",
      });
    } catch {}
  }
}
