export type ThemeName = "light" | "sepia" | "dark";
export type BookFormat = "epub" | "pdf" | "mobi" | string;

export interface Theme {
  name: ThemeName;
  bg: string;
  fg: string;
  label: string;
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

export type FontFamily = "serif" | "sans" | "mono";
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
