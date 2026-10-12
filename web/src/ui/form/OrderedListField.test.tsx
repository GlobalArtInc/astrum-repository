import { fireEvent, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderWithApp } from '../../test/render';
import { addItem, moveItem, OrderedListField, removeItem } from './OrderedListField';

vi.setConfig({ testTimeout: 30_000 });

describe('ordering helpers', () => {
  it('move, add and remove items', () => {
    expect(moveItem(['a', 'b', 'c'], 2, 0)).toEqual(['c', 'a', 'b']);
    expect(moveItem(['a', 'b', 'c'], 0, 1)).toEqual(['b', 'a', 'c']);
    expect(moveItem(['a', 'b'], 0, 5)).toEqual(['a', 'b']);
    expect(moveItem(['a', 'b'], -1, 0)).toEqual(['a', 'b']);
    expect(addItem(['a'], 'b')).toEqual(['a', 'b']);
    expect(addItem(['a'], 'a')).toEqual(['a']);
    expect(removeItem(['a', 'b'], 'a')).toEqual(['b']);
  });
});

function Members({ disabled = false }: { disabled?: boolean }) {
  const [value, setValue] = useState(['one', 'two', 'three']);
  return (
    <>
      <OrderedListField
        label="Members"
        value={value}
        onChange={setValue}
        disabled={disabled}
        options={['one', 'two', 'three', 'four'].map((name) => ({ value: name, label: name.toUpperCase() }))}
        addPlaceholder="Add a member"
      />
      <output>{value.join(',')}</output>
    </>
  );
}

function order() {
  return screen.getByRole('status').textContent;
}

describe('the ordered list field', () => {
  it('reorders with buttons and by dragging', async () => {
    renderWithApp(<Members />);
    const list = screen.getByRole('list', { name: 'Members' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.getByRole('button', { name: 'Move ONE up' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Move THREE down' })).toBeDisabled();
    await userEvent.click(screen.getByRole('button', { name: 'Move ONE down' }));
    expect(order()).toBe('two,one,three');
    await userEvent.click(screen.getByRole('button', { name: 'Move THREE up' }));
    expect(order()).toBe('two,three,one');
    const items = within(list).getAllByRole('listitem');
    fireEvent.dragStart(items[2] as HTMLElement, { dataTransfer: { setData() {}, effectAllowed: '' } });
    fireEvent.dragOver(items[0] as HTMLElement);
    fireEvent.drop(items[0] as HTMLElement);
    expect(order()).toBe('one,two,three');
  });

  it('adds what is not chosen yet and removes members', async () => {
    renderWithApp(<Members />);
    await userEvent.click(screen.getByRole('button', { name: 'Remove TWO' }));
    expect(order()).toBe('one,three');
    await userEvent.click(screen.getByRole('combobox', { name: 'Add a member' }));
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual(['TWO', 'FOUR']);
    await userEvent.click(screen.getByRole('option', { name: 'FOUR' }));
    expect(order()).toBe('one,three,four');
  });

  it('only shows the order when disabled', () => {
    renderWithApp(<Members disabled />);
    expect(screen.queryByRole('button', { name: /Move/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
    expect(within(screen.getByRole('list', { name: 'Members' })).getAllByRole('listitem')).toHaveLength(3);
  });
});
