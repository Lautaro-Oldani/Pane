// Tipos compartidos que coinciden con las structs de Rust

export interface Clip {
  id: number;
  content: string;
  content_type: "text" | "image" | "url" | "code" | "color" | "html";
  preview: string | null;
  image_base64: string | null;
  source_app: string | null;
  hash: string;
  is_pinned: boolean;
  is_favorite: boolean;
  collection_id: number | null;
  created_at: string;
}

export interface Collection {
  id: number;
  name: string;
  icon: string | null;
  created_at: string;
}

export type FilterType = "all" | "pinned" | "favorites" | "text" | "image" | "url" | "code" | "color" | "html";
