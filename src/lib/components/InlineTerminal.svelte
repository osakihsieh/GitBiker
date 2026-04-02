<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { app } from '$lib/stores/app.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';

  interface Props {
    visible: boolean;
    onClose: () => void;
  }

  let { visible, onClose }: Props = $props();

  let terminalEl = $state<HTMLDivElement | null>(null);
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let inputBuffer = '';
  let mounted = false;

  interface ShellOutput {
    stdout: string;
    stderr: string;
    exit_code: number;
  }

  function initTerminal() {
    if (!terminalEl || terminal) return;

    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: 'var(--font-mono), "Cascadia Code", "Fira Code", monospace',
      theme: {
        background: '#1e1e2e',
        foreground: '#cdd6f4',
        cursor: '#f5e0dc',
        selectionBackground: '#45475a',
        black: '#45475a',
        red: '#f38ba8',
        green: '#a6e3a1',
        yellow: '#f9e2af',
        blue: '#89b4fa',
        magenta: '#f5c2e7',
        cyan: '#94e2d5',
        white: '#bac2de',
      },
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalEl);
    fitAddon.fit();

    writePrompt();

    terminal.onData((data) => {
      if (data === '\r') {
        // Enter
        terminal!.writeln('');
        const cmd = inputBuffer.trim();
        inputBuffer = '';
        if (cmd) {
          executeCommand(cmd);
        } else {
          writePrompt();
        }
      } else if (data === '\x7f') {
        // Backspace
        if (inputBuffer.length > 0) {
          inputBuffer = inputBuffer.slice(0, -1);
          terminal!.write('\b \b');
        }
      } else if (data === '\x03') {
        // Ctrl+C
        inputBuffer = '';
        terminal!.writeln('^C');
        writePrompt();
      } else if (data >= ' ') {
        // Printable character
        inputBuffer += data;
        terminal!.write(data);
      }
    });

    mounted = true;
  }

  function writePrompt() {
    const repoName = app.repoName || 'gitbiker';
    const branch = app.currentBranch || 'main';
    terminal?.write(`\x1b[36m${repoName}\x1b[0m \x1b[33m(${branch})\x1b[0m $ `);
  }

  async function executeCommand(cmd: string) {
    if (!app.repoPath) {
      terminal?.writeln('\x1b[31mNo repository open\x1b[0m');
      writePrompt();
      return;
    }

    // Handle built-in commands
    if (cmd === 'clear' || cmd === 'cls') {
      terminal?.clear();
      writePrompt();
      return;
    }
    if (cmd === 'exit') {
      onClose();
      return;
    }

    try {
      const result: ShellOutput = await invoke('run_shell_command', {
        path: app.repoPath,
        command: cmd,
      });

      if (result.stdout) {
        terminal?.writeln(result.stdout.trimEnd());
      }
      if (result.stderr) {
        const color = result.exit_code === 0 ? '33' : '31'; // yellow for warnings, red for errors
        terminal?.writeln(`\x1b[${color}m${result.stderr.trimEnd()}\x1b[0m`);
      }

      // Auto refresh git status after git commands
      if (cmd.startsWith('git ')) {
        app.refreshAll().catch(() => {});
      }
    } catch (e: unknown) {
      terminal?.writeln(`\x1b[31m${String(e)}\x1b[0m`);
    }

    writePrompt();
  }

  $effect(() => {
    if (visible && terminalEl && !mounted) {
      // Wait for DOM to settle
      requestAnimationFrame(() => initTerminal());
    }
  });

  $effect(() => {
    if (visible && fitAddon) {
      requestAnimationFrame(() => fitAddon?.fit());
    }
  });

  onMount(() => {
    const observer = new ResizeObserver(() => {
      if (visible && fitAddon) fitAddon.fit();
    });
    if (terminalEl) observer.observe(terminalEl);
    return () => observer.disconnect();
  });

  onDestroy(() => {
    terminal?.dispose();
    terminal = null;
    fitAddon = null;
    mounted = false;
  });
</script>

{#if visible}
  <div class="terminal-panel">
    <div class="terminal-header">
      <span class="terminal-title">Terminal (git commands only)</span>
      <button class="terminal-close" onclick={onClose}>×</button>
    </div>
    <div class="terminal-body" bind:this={terminalEl}></div>
  </div>
{/if}

<style>
  .terminal-panel {
    border-top: 1px solid var(--border);
    background: #1e1e2e;
    display: flex;
    flex-direction: column;
    height: 200px;
    min-height: 100px;
    flex-shrink: 0;
  }
  .terminal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px var(--space-md);
    background: #181825;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    flex-shrink: 0;
  }
  .terminal-title {
    font-size: 11px;
    color: #6c7086;
    font-family: var(--font-ui);
  }
  .terminal-close {
    background: none;
    border: none;
    color: #6c7086;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 3px;
  }
  .terminal-close:hover { color: #cdd6f4; background: rgba(255, 255, 255, 0.05); }
  .terminal-body {
    flex: 1;
    padding: var(--space-xs);
    overflow: hidden;
  }

  .terminal-body :global(.xterm) {
    height: 100%;
  }
  .terminal-body :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
