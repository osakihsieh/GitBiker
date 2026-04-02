import { describe, it, expect, vi } from 'vitest';
import { clickOutside } from './clickOutside';

describe('clickOutside', () => {
  it('calls callback on mousedown outside the node', () => {
    const node = document.createElement('div');
    document.body.appendChild(node);
    const callback = vi.fn();

    const action = clickOutside(node, callback);

    // Click outside
    const event = new MouseEvent('mousedown', { bubbles: true });
    document.body.dispatchEvent(event);

    expect(callback).toHaveBeenCalledTimes(1);

    action.destroy();
    document.body.removeChild(node);
  });

  it('does not call callback on mousedown inside the node', () => {
    const node = document.createElement('div');
    const child = document.createElement('span');
    node.appendChild(child);
    document.body.appendChild(node);
    const callback = vi.fn();

    const action = clickOutside(node, callback);

    const event = new MouseEvent('mousedown', { bubbles: true });
    child.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();

    action.destroy();
    document.body.removeChild(node);
  });

  it('calls callback on Escape key', () => {
    const node = document.createElement('div');
    document.body.appendChild(node);
    const callback = vi.fn();

    const action = clickOutside(node, callback);

    const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
    document.dispatchEvent(event);

    expect(callback).toHaveBeenCalledTimes(1);

    action.destroy();
    document.body.removeChild(node);
  });

  it('does not call callback on non-Escape key', () => {
    const node = document.createElement('div');
    document.body.appendChild(node);
    const callback = vi.fn();

    const action = clickOutside(node, callback);

    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
    document.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();

    action.destroy();
    document.body.removeChild(node);
  });

  it('removes listeners on destroy', () => {
    const node = document.createElement('div');
    document.body.appendChild(node);
    const callback = vi.fn();

    const action = clickOutside(node, callback);
    action.destroy();

    const event = new MouseEvent('mousedown', { bubbles: true });
    document.body.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();
    document.body.removeChild(node);
  });

  it('supports callback update', () => {
    const node = document.createElement('div');
    document.body.appendChild(node);
    const callback1 = vi.fn();
    const callback2 = vi.fn();

    const action = clickOutside(node, callback1);
    action.update(callback2);

    const event = new MouseEvent('mousedown', { bubbles: true });
    document.body.dispatchEvent(event);

    expect(callback1).not.toHaveBeenCalled();
    expect(callback2).toHaveBeenCalledTimes(1);

    action.destroy();
    document.body.removeChild(node);
  });
});
