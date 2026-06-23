export type ThemeName =
  | "light"
  | "sepia"
  | "dark"
  | "cream"
  | "green"
  | "night";
export type BookFormat = "epub" | "mobi" | string;
export type HighlightColor = "yellow" | "green" | "blue" | "pink";

export interface Theme {
  name: ThemeName;
  bg: string;
  fg: string;
  label: string;
  isDark: boolean;
}

export interface Book {
  id: string;
  title: string;
  author: string | null;
  format: BookFormat;
  cover_path: string | null;
  file_size: number | null;
  page_count: number | null;
  current_page: string | null;
  reading_status: string | null;
  last_opened_at: string | null;
  progress: number | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface TagCount {
  name: string;
  count: number;
}

export interface Stats {
  total_books: number;
  finished_books: number;
  reading_books: number;
  want_to_read: number;
  total_highlights: number;
}

export interface TocItem {
  label: string;
  href: string;
  subitems: TocItem[];
}

export interface FlattenedTocItem {
  label: string;
  href: string;
  index: number;
}

export type FontFamily =
  | "literata"
  | "andika"
  | "shantell"
  | "noto"
  | "libertinus";
export type Alignment = "left" | "justify";

export interface Typography {
  fontFamily: FontFamily;
  fontSize: number;
  lineHeight: number;
  margin: number;
  align: Alignment;
}

export interface ReaderSettings {
  theme: ThemeName;
  typography: Typography;
  cfi?: string;
}

export interface Highlight {
  id: string;
  chapterIndex: number;
  chapterLabel: string | null;
  cfiRange: string;
  text: string;
  color: HighlightColor;
  note: string | null;
  createdAt: number;
}

export interface Bookmark {
  id: string;
  chapterIndex: number;
  chapterLabel: string | null;
  cfi: string;
  createdAt: number;
}

export interface SpineItem {
  href: string;
  id: string;
  media_type: string;
}

export interface SearchResult {
  id: string;
  chapterIndex: number;
  chapterLabel: string;
  excerpt: string;
  href: string;
}
