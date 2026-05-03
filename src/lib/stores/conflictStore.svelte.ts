import { app } from './app.svelte';
import { gitGetConflictFiles, gitGetConflictContent } from '$lib/git/commands';
import { extractErrorMessage } from '$lib/utils/error';
import type { ConflictFile, ConflictContent } from '$lib/git/types';

class ConflictStore {
  get files(): ConflictFile[] {
    return app.activeTab?.state.conflictFiles ?? [];
  }

  get activeFile(): string | null {
    return app.activeTab?.state.activeConflictFile ?? null;
  }

  get content(): ConflictContent | null {
    return app.activeTab?.state.conflictContent ?? null;
  }

  get hunkChoices(): Record<number, 'Ours' | 'Theirs' | 'Both' | 'Custom'> {
    return app.activeTab?.state.hunkChoices ?? {};
  }

  get isInConflictMode(): boolean {
    return app.viewMode === 'conflict-resolution';
  }

  async enterConflictMode(): Promise<void> {
    const path = app.repoPath;
    if (!path) return;

    try {
      const files = await gitGetConflictFiles(path);
      const tab = app.activeTab;
      if (!tab) return;
      tab.state.conflictFiles = files;
      tab.state.activeConflictFile = files.length > 0 ? files[0].path : null;
      tab.state.conflictContent = null;
      tab.state.hunkChoices = {};
      tab.state.viewMode = 'conflict-resolution';

      if (files.length > 0) {
        await this.selectConflictFile(files[0].path);
      }
    } catch (e: unknown) {
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  exitConflictMode(): void {
    const tab = app.activeTab;
    if (!tab) return;
    tab.state.conflictFiles = [];
    tab.state.activeConflictFile = null;
    tab.state.conflictContent = null;
    tab.state.hunkChoices = {};
    tab.state.viewMode = 'worktree';
    app.selectedCommit = null;
  }

  async selectConflictFile(filePath: string): Promise<void> {
    const path = app.repoPath;
    const tab = app.activeTab;
    if (!path || !tab) return;

    tab.state.activeConflictFile = filePath;
    tab.state.hunkChoices = {};

    try {
      const content = await gitGetConflictContent(path, filePath);
      tab.state.conflictContent = content;
    } catch (e: unknown) {
      tab.state.conflictContent = null;
      app.addToast(extractErrorMessage(e), 'error');
    }
  }

  setHunkChoice(hunkIndex: number, choice: 'Ours' | 'Theirs' | 'Both' | 'Custom'): void {
    const tab = app.activeTab;
    if (!tab) return;
    tab.state.hunkChoices = { ...tab.state.hunkChoices, [hunkIndex]: choice };
  }

  async refreshConflictFiles(): Promise<void> {
    const path = app.repoPath;
    const tab = app.activeTab;
    if (!path || !tab) return;

    try {
      const files = await gitGetConflictFiles(path);
      tab.state.conflictFiles = files;
    } catch {
      this.exitConflictMode();
    }
  }
}

export const conflicts = new ConflictStore();
