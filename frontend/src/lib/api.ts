const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";

export async function api(path: string, init?: RequestInit): Promise<Response> {
  const headers: Record<string, string> = {
    ...(init?.headers as Record<string, string>),
  };
  if (!(init?.body instanceof FormData)) {
    headers["Content-Type"] = "application/json";
  }
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    credentials: "include",
    headers,
  });
  return res;
}

export function uploadFile(
  path: string,
  formData: FormData,
  onProgress?: (loaded: number, total: number) => void,
): Promise<Response> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", `${API_URL}${path}`);
    xhr.withCredentials = true;
    xhr.responseType = "text";

    if (onProgress) {
      xhr.upload.addEventListener("progress", (e) => {
        if (e.lengthComputable) {
          onProgress(e.loaded, e.total);
        }
      });
    }

    xhr.addEventListener("load", () => {
      const headers = parseHeaders(xhr.getAllResponseHeaders());
      const body = new Blob([xhr.response ?? ""], { type: headers.get("content-type") ?? "application/json" });
      resolve(new Response(body, { status: xhr.status, statusText: xhr.statusText, headers }));
    });

    xhr.addEventListener("error", () => {
      reject(new Error("Network error"));
    });

    xhr.addEventListener("abort", () => {
      reject(new Error("Upload aborted"));
    });

    xhr.send(formData);
  });
}

function parseHeaders(raw: string): Headers {
  const headers = new Headers();
  for (const line of raw.split(/[\r\n]+/)) {
    if (!line) continue;
    const idx = line.indexOf(":");
    if (idx < 0) continue;
    const key = line.slice(0, idx).trim();
    const value = line.slice(idx + 1).trim();
    if (key) headers.set(key, value);
  }
  return headers;
}

export async function extractErrorMessage(res: Response): Promise<string> {
  try {
    const data = await res.clone().json();
    if (data && typeof data.error === "string") return data.error;
  } catch {}
  return `HTTP ${res.status} ${res.statusText}`;
}

export async function fetchStats(): Promise<import("./reader/types").Stats | null> {
  try {
    const res = await api("/api/stats");
    if (res.ok) return await res.json();
  } catch {}
  return null;
}

export async function fetchTags(): Promise<import("./reader/types").TagCount[]> {
  try {
    const res = await api("/api/tags");
    if (res.ok) return await res.json();
  } catch {}
  return [];
}

export async function setBookTags(bookId: string, tags: string[]): Promise<string[]> {
  const res = await api(`/api/books/${bookId}/tags`, {
    method: "PUT",
    body: JSON.stringify(tags),
  });
  if (res.ok) return await res.json();
  return [];
}

export async function updateBook(bookId: string, payload: Record<string, unknown>): Promise<Response> {
  return api(`/api/books/${bookId}`, {
    method: "PATCH",
    body: JSON.stringify(payload),
  });
}

export async function touchBook(bookId: string): Promise<void> {
  try {
    await api(`/api/books/${bookId}/touch`, { method: "POST" });
  } catch {}
}

export async function fetchBooks(params?: Record<string, string>): Promise<import("./reader/types").Book[]> {
  const qs = params ? "?" + new URLSearchParams(params).toString() : "";
  try {
    const res = await api(`/api/books${qs}`);
    if (res.ok) return await res.json();
  } catch {}
  return [];
}
