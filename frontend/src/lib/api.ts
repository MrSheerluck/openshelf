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
