/**
 * 從各種型別的 error 值提取可讀的錯誤訊息字串。
 * Tauri invoke 拋出的 error 通常是 Rust 序列化的字串或物件，
 * 不是 JavaScript 的 Error 實例。
 */
export function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  if (error !== null && typeof error === 'object') {
    const obj = error as Record<string, unknown>;
    if (typeof obj['message'] === 'string') return obj['message'];
    if (typeof obj['error'] === 'string') return obj['error'];
    if (typeof obj['msg'] === 'string') return obj['msg'];
    const json = JSON.stringify(error, null, 2);
    if (json !== '{}') return json;
  }
  return String(error);
}
