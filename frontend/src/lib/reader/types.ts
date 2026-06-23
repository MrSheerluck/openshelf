export type ThemeName = "light" | "sepia" | "dark" | "cream" | "green" | "night";
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

export type FontFamily = "literata" | "andika" | "shantell" | "noto" | "libertinus";
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
