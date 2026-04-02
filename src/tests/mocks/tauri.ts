import { vi } from 'vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/plugin-store
const mockStore = {
  get: vi.fn(),
  set: vi.fn(),
  save: vi.fn(),
  delete: vi.fn(),
  has: vi.fn(),
  clear: vi.fn(),
};

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn().mockResolvedValue(mockStore),
  Store: vi.fn(),
}));

export { mockStore };
