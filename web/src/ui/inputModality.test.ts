import { afterEach, describe, expect, it } from 'vitest';
import { modalityOf, watchInputModality } from './inputModality';

const plain = { metaKey: false, ctrlKey: false, altKey: false };

describe('the input modality', () => {
  let stop: (() => void) | undefined;
  afterEach(() => {
    stop?.();
    delete document.documentElement.dataset.input;
  });

  it('counts only keys that move or act, not modifiers or shortcuts', () => {
    for (const key of ['Tab', 'ArrowDown', 'Enter', ' ', 'Escape', 'a']) {
      expect(modalityOf({ key, ...plain }), key).toBe('keyboard');
    }
    for (const key of ['Shift', 'Meta', 'Control', 'Alt', 'CapsLock']) {
      expect(modalityOf({ key, ...plain }), key).toBeNull();
    }
    expect(modalityOf({ key: 'c', ...plain, metaKey: true })).toBeNull();
    expect(modalityOf({ key: 'k', ...plain, ctrlKey: true })).toBeNull();
    expect(modalityOf({ key: 'Tab', ...plain })).toBe('keyboard');
  });

  it('marks the document after a pointer press and after a real key', () => {
    stop = watchInputModality();
    const root = document.documentElement;
    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    expect(root.dataset.input).toBe('pointer');
    document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Shift', bubbles: true }));
    expect(root.dataset.input).toBe('pointer');
    document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
    expect(root.dataset.input).toBe('keyboard');
  });
});
