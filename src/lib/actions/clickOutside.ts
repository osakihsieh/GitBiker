/**
 * Svelte action: close on click outside or Escape key.
 * Usage: <div use:clickOutside={onClose}>...</div>
 */
export function clickOutside(node: HTMLElement, callback: () => void) {
  function handleClick(e: MouseEvent) {
    if (!node.contains(e.target as Node)) {
      callback();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      callback();
    }
  }

  document.addEventListener('mousedown', handleClick, true);
  document.addEventListener('keydown', handleKeydown, true);

  return {
    update(newCallback: () => void) {
      callback = newCallback;
    },
    destroy() {
      document.removeEventListener('mousedown', handleClick, true);
      document.removeEventListener('keydown', handleKeydown, true);
    },
  };
}
