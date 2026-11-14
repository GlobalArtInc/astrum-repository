import { PROMOTE_LIMIT, type PromotionItem } from './model';

export interface PromoteEntry {
  id: string;
  source: string;
  format: string;
  item: PromotionItem;
}

export function atLimit(selected: readonly PromoteEntry[]): boolean {
  return selected.length >= PROMOTE_LIMIT;
}

export function isChosen(selected: readonly PromoteEntry[], id: string): boolean {
  return selected.some((entry) => entry.id === id);
}

export function canPick(selected: readonly PromoteEntry[], entry: PromoteEntry): boolean {
  return selected.every((chosen) => chosen.source === entry.source);
}

export function toggleSelection(selected: readonly PromoteEntry[], entry: PromoteEntry): PromoteEntry[] {
  if (isChosen(selected, entry.id)) {
    return selected.filter((chosen) => chosen.id !== entry.id);
  }
  return atLimit(selected) || !canPick(selected, entry) ? [...selected] : [...selected, entry];
}

export function selectPage(
  selected: readonly PromoteEntry[],
  entries: readonly PromoteEntry[],
): PromoteEntry[] {
  const next = [...selected];
  for (const entry of entries) {
    if (atLimit(next)) {
      break;
    }
    if (!isChosen(next, entry.id) && canPick(next, entry)) {
      next.push(entry);
    }
  }
  return next;
}
