import { describe, expect, it } from 'vitest';
import { PROMOTE_LIMIT } from './model';
import { type PromoteEntry, selectPage, toggleSelection } from './selection';

function entry(id: string, source = 'staging'): PromoteEntry {
  return { id, source, format: 'maven2', item: { componentId: id } };
}

function entries(count: number, source = 'staging'): PromoteEntry[] {
  return Array.from({ length: count }, (_, index) => entry(`item-${index}`, source));
}

describe('the promotion selection', () => {
  it('checks and unchecks an item', () => {
    const checked = toggleSelection([], entry('a'));
    expect(checked.map((chosen) => chosen.id)).toEqual(['a']);
    expect(toggleSelection(checked, entry('a'))).toEqual([]);
  });

  it('refuses a check past the limit of items per request', () => {
    const full = entries(PROMOTE_LIMIT);
    expect(toggleSelection(full, entry('extra'))).toEqual(full);
  });

  it('refuses an item from another repository than the one already chosen', () => {
    const chosen = [entry('a', 'staging')];
    expect(toggleSelection(chosen, entry('b', 'other'))).toEqual(chosen);
  });

  it('selects the page up to the limit, keeping what was selected', () => {
    const selected = [entry('kept')];
    const next = selectPage(selected, entries(PROMOTE_LIMIT));
    expect(next).toHaveLength(PROMOTE_LIMIT);
    expect(next[0]?.id).toBe('kept');
  });
});
