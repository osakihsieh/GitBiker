import { vi } from 'vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/api/event
const mockUnlisten = vi.fn();
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(mockUnlisten),
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

// Mock git commands
const mockGitCommands = {
  gitStatus: vi.fn().mockResolvedValue([]),
  gitLog: vi.fn().mockResolvedValue([]),
  gitBranches: vi.fn().mockResolvedValue([]),
  gitDiff: vi.fn().mockResolvedValue(null),
  gitMergeBranch: vi.fn().mockResolvedValue({ branch: '', success: true, message: '', conflicts: [] }),
  gitMergeAbort: vi.fn().mockResolvedValue(undefined),
  gitStashList: vi.fn().mockResolvedValue([]),
  gitStashPush: vi.fn().mockResolvedValue(''),
  gitStashPop: vi.fn().mockResolvedValue(''),
  gitStashApply: vi.fn().mockResolvedValue(''),
  gitStashDrop: vi.fn().mockResolvedValue(''),
  startWatching: vi.fn().mockResolvedValue(undefined),
  stopWatching: vi.fn().mockResolvedValue(undefined),
};

vi.mock('$lib/git/commands', () => mockGitCommands);

export { mockStore, mockUnlisten, mockGitCommands };
