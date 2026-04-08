import { gitStatus, gitBranches, gitFetch, gitPull, gitPush, scanGitRepos } from '$lib/git/commands';
import { repoNameFromPath } from './app.svelte';
import {
  loadScanPaths,
  saveScanPaths,
  loadCachedRepoInfos,
  saveCachedRepoInfos,
  loadMultiRepoRefreshInterval,
  type CachedRepoInfo,
} from './persistence.svelte';
import type { Branch } from '$lib/git/types';

// ── Types ──────────────────────────────────────────────

export interface RepoInfo {
  path: string;
  name: string;
  branch: string;
  dirty: number;
  ahead: number;
  behind: number;
  loading: boolean;
  error: string | null;
  scanPath: string;
}

// ── Store ──────────────────────────────────────────────

class MultiRepoStore {
  scanPaths = $state<string[]>([]);
  repos = $state<RepoInfo[]>([]);
  loading = $state(false);
  bulkRunning = $state(false);
  bulkAction = $state('');
  refreshIntervalSec = $state(60);

  private refreshTimer: ReturnType<typeof setInterval> | null = null;
  private initialized = false;

  // ── Computed ──

  get dirtyCount(): number {
    return this.repos.filter((r) => r.dirty > 0 && !r.error).length;
  }

  get pushableCount(): number {
    return this.repos.filter((r) => r.ahead > 0 && !r.error).length;
  }

  get hasScanPaths(): boolean {
    return this.scanPaths.length > 0;
  }

  get repoCount(): number {
    return this.repos.length;
  }

  // ── Grouped repos (smart grouping: 1 dir = flat, 2+ = grouped) ──

  get groupedRepos(): { scanPath: string; repos: RepoInfo[] }[] {
    const groups = new Map<string, RepoInfo[]>();
    for (const repo of this.repos) {
      const group = groups.get(repo.scanPath) ?? [];
      group.push(repo);
      groups.set(repo.scanPath, group);
    }
    return Array.from(groups.entries()).map(([scanPath, repos]) => ({ scanPath, repos }));
  }

  get useGrouping(): boolean {
    return this.scanPaths.length > 1;
  }

  // ── Initialization ──

  async init(activeRepoPath?: string | null) {
    if (this.initialized) return;
    this.initialized = true;

    // Load persisted data
    const [paths, cached, interval] = await Promise.all([
      loadScanPaths(),
      loadCachedRepoInfos(),
      loadMultiRepoRefreshInterval(),
    ]);

    this.scanPaths = paths;
    this.refreshIntervalSec = interval;

    // Show cached data immediately
    if (cached.length > 0) {
      this.repos = cached.map((c) => ({
        ...c,
        loading: false,
        error: null,
      }));
    }

    // Background refresh
    if (paths.length > 0) {
      this.refreshAll(activeRepoPath);
    }

    // Start periodic refresh
    this.startRefreshTimer(activeRepoPath);
  }

  // ── Scan Path Management ──

  async addScanPath(path: string, activeRepoPath?: string | null): Promise<void> {
    // Exact duplicate check
    const normalized = path.replace(/\\/g, '/');
    if (this.scanPaths.some((p) => p.replace(/\\/g, '/') === normalized)) {
      return; // 已存在，靜默忽略
    }

    // Subdirectory overlap check (warn but allow)
    const isSubdir = this.scanPaths.some((existing) => {
      const e = existing.replace(/\\/g, '/');
      return normalized.startsWith(e + '/') || e.startsWith(normalized + '/');
    });

    this.scanPaths = [...this.scanPaths, path];
    await saveScanPaths(this.scanPaths);

    // Scan the new path
    try {
      const repoPaths = await scanGitRepos(path);
      if (repoPaths.length === 0) {
        return;
      }
      const newRepos = await Promise.all(
        repoPaths.map((rp) => this.loadRepoInfo(rp, path)),
      );
      // Merge, avoiding duplicates
      const existingPaths = new Set(this.repos.map((r) => r.path));
      const unique = newRepos.filter((r) => !existingPaths.has(r.path));
      this.repos = [...this.repos, ...unique];
      await this.persistCache();
    } catch {
      // scan failed — path stays in scanPaths, repos empty for this path
    }

    return; // isSubdir warning handled by caller if needed
  }

  async removeScanPath(path: string): Promise<void> {
    this.scanPaths = this.scanPaths.filter((p) => p !== path);
    this.repos = this.repos.filter((r) => r.scanPath !== path);
    await Promise.all([saveScanPaths(this.scanPaths), this.persistCache()]);
  }

  // ── Load / Refresh ──

  async refreshAll(activeRepoPath?: string | null): Promise<void> {
    if (this.scanPaths.length === 0) return;
    this.loading = true;

    try {
      // Re-scan all paths
      const allRepoPaths: { path: string; scanPath: string }[] = [];
      await Promise.all(
        this.scanPaths.map(async (sp) => {
          try {
            const paths = await scanGitRepos(sp);
            for (const p of paths) {
              allRepoPaths.push({ path: p, scanPath: sp });
            }
          } catch {
            // Skip failed scan paths — they may have been deleted
          }
        }),
      );

      // Remove stale scan paths (no repos found and path doesn't exist)
      // Keep scan paths that exist even if empty

      // Load info for each repo (skip active repo to avoid timer race)
      const infos = await Promise.all(
        allRepoPaths.map(({ path: rp, scanPath: sp }) => {
          if (activeRepoPath && rp.replace(/\\/g, '/') === activeRepoPath.replace(/\\/g, '/')) {
            // Return existing cached info for active repo
            const existing = this.repos.find((r) => r.path === rp);
            if (existing) return Promise.resolve(existing);
          }
          return this.loadRepoInfo(rp, sp);
        }),
      );

      this.repos = infos;
      await this.persistCache();
    } finally {
      this.loading = false;
    }
  }

  private async loadRepoInfo(repoPath: string, scanPath: string): Promise<RepoInfo> {
    try {
      const [status, branches] = await Promise.all([
        gitStatus(repoPath),
        gitBranches(repoPath),
      ]);
      const current = branches.find((b: Branch) => b.is_current);
      return {
        path: repoPath,
        name: repoNameFromPath(repoPath),
        branch: current?.name ?? 'unknown',
        dirty: status.length,
        ahead: current?.ahead ?? 0,
        behind: current?.behind ?? 0,
        loading: false,
        error: null,
        scanPath,
      };
    } catch (e: unknown) {
      return {
        path: repoPath,
        name: repoNameFromPath(repoPath),
        branch: '?',
        dirty: 0,
        ahead: 0,
        behind: 0,
        loading: false,
        error: String(e),
        scanPath,
      };
    }
  }

  // ── Bulk Operations ──

  async bulkFetch(): Promise<{ ok: number; fail: number }> {
    if (this.bulkRunning || this.repos.length === 0) return { ok: 0, fail: 0 };
    this.bulkRunning = true;
    this.bulkAction = 'Fetching...';
    try {
      const results = await Promise.allSettled(
        this.repos.map((r) => gitFetch(r.path)),
      );
      const ok = results.filter((r) => r.status === 'fulfilled').length;
      const fail = results.filter((r) => r.status === 'rejected').length;
      await this.refreshAll();
      return { ok, fail };
    } finally {
      this.bulkRunning = false;
      this.bulkAction = '';
    }
  }

  async bulkPull(): Promise<{ ok: number; fail: number }> {
    if (this.bulkRunning || this.repos.length === 0) return { ok: 0, fail: 0 };
    this.bulkRunning = true;
    this.bulkAction = 'Pulling...';
    try {
      const results = await Promise.allSettled(
        this.repos.map((r) => gitPull(r.path)),
      );
      const ok = results.filter((r) => r.status === 'fulfilled').length;
      const fail = results.filter((r) => r.status === 'rejected').length;
      await this.refreshAll();
      return { ok, fail };
    } finally {
      this.bulkRunning = false;
      this.bulkAction = '';
    }
  }

  async bulkPush(): Promise<{ ok: number; fail: number }> {
    if (this.bulkRunning) return { ok: 0, fail: 0 };
    const pushable = this.repos.filter((r) => r.ahead > 0);
    if (pushable.length === 0) return { ok: 0, fail: 0 };
    this.bulkRunning = true;
    this.bulkAction = 'Pushing...';
    try {
      const results = await Promise.allSettled(
        pushable.map((r) => gitPush(r.path)),
      );
      const ok = results.filter((r) => r.status === 'fulfilled').length;
      const fail = results.filter((r) => r.status === 'rejected').length;
      await this.refreshAll();
      return { ok, fail };
    } finally {
      this.bulkRunning = false;
      this.bulkAction = '';
    }
  }

  // ── Single Repo Actions ──

  async repoFetch(repoPath: string): Promise<void> {
    const repo = this.repos.find((r) => r.path === repoPath);
    if (!repo) return;
    repo.loading = true;
    try {
      await gitFetch(repoPath);
      const updated = await this.loadRepoInfo(repoPath, repo.scanPath);
      this.repos = this.repos.map((r) => (r.path === repoPath ? updated : r));
      await this.persistCache();
    } catch (e: unknown) {
      repo.error = String(e);
      repo.loading = false;
    }
  }

  async repoPull(repoPath: string): Promise<void> {
    const repo = this.repos.find((r) => r.path === repoPath);
    if (!repo) return;
    repo.loading = true;
    try {
      await gitPull(repoPath);
      const updated = await this.loadRepoInfo(repoPath, repo.scanPath);
      this.repos = this.repos.map((r) => (r.path === repoPath ? updated : r));
      await this.persistCache();
    } catch (e: unknown) {
      repo.error = String(e);
      repo.loading = false;
    }
  }

  async repoPush(repoPath: string): Promise<void> {
    const repo = this.repos.find((r) => r.path === repoPath);
    if (!repo) return;
    repo.loading = true;
    try {
      await gitPush(repoPath);
      const updated = await this.loadRepoInfo(repoPath, repo.scanPath);
      this.repos = this.repos.map((r) => (r.path === repoPath ? updated : r));
      await this.persistCache();
    } catch (e: unknown) {
      repo.error = String(e);
      repo.loading = false;
    }
  }

  // ── Timer ──

  private startRefreshTimer(activeRepoPath?: string | null) {
    this.stopRefreshTimer();
    if (this.refreshIntervalSec <= 0) return;
    this.refreshTimer = setInterval(() => {
      this.refreshAll(activeRepoPath);
    }, this.refreshIntervalSec * 1000);
  }

  stopRefreshTimer() {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  updateActiveRepoPath(activeRepoPath: string | null) {
    // Restart timer with updated active repo to skip
    this.stopRefreshTimer();
    this.startRefreshTimer(activeRepoPath);
  }

  // ── Persistence ──

  private async persistCache(): Promise<void> {
    const cached: CachedRepoInfo[] = this.repos
      .filter((r) => !r.error)
      .map((r) => ({
        path: r.path,
        name: r.name,
        branch: r.branch,
        dirty: r.dirty,
        ahead: r.ahead,
        behind: r.behind,
        scanPath: r.scanPath,
      }));
    await saveCachedRepoInfos(cached);
  }

  // ── Search ──

  filterRepos(query: string): RepoInfo[] {
    if (!query.trim()) return this.repos;
    const q = query.toLowerCase();
    return this.repos.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.branch.toLowerCase().includes(q) ||
        r.path.toLowerCase().includes(q),
    );
  }

  // ── Cleanup ──

  destroy() {
    this.stopRefreshTimer();
  }
}

export const multiRepo = new MultiRepoStore();
