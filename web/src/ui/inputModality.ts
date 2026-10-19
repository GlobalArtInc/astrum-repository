const MODIFIERS = new Set([
  'Alt',
  'AltGraph',
  'CapsLock',
  'Control',
  'Fn',
  'FnLock',
  'Hyper',
  'Meta',
  'OS',
  'Shift',
  'Super',
]);

export type Modality = 'keyboard' | 'pointer';

export function currentModality(): Modality | undefined {
  const value = document.documentElement.dataset.input;
  return value === 'keyboard' || value === 'pointer' ? value : undefined;
}

export function modalityOf(
  event: Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey'>,
): Modality | null {
  if (MODIFIERS.has(event.key) || event.metaKey || event.ctrlKey || event.altKey) {
    return null;
  }
  return 'keyboard';
}

export function watchInputModality(root: HTMLElement = document.documentElement): () => void {
  const onPointer = () => {
    root.dataset.input = 'pointer';
  };
  const onKey = (event: KeyboardEvent) => {
    const modality = modalityOf(event);
    if (modality) {
      root.dataset.input = modality;
    }
  };
  document.addEventListener('pointerdown', onPointer, true);
  document.addEventListener('keydown', onKey, true);
  return () => {
    document.removeEventListener('pointerdown', onPointer, true);
    document.removeEventListener('keydown', onKey, true);
  };
}
