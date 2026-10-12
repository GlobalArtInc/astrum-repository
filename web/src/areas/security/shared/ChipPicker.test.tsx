import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../../../test/render';
import { ChipPicker, matchOptions, type PickerOption } from './ChipPicker';

const OPTIONS: PickerOption[] = Array.from({ length: 8 }, (_, index) => ({
  value: `role-${index}`,
  label: `role-${index}`,
  detail: index === 3 ? 'Deployers' : undefined,
}));

function Harness({ initial = [] as string[] }) {
  const [value, setValue] = useState(initial);
  return <ChipPicker label="Roles" value={value} onChange={setValue} options={OPTIONS} limit={5} />;
}

describe('the chip picker', () => {
  it('matches every word in value, label and detail', () => {
    expect(matchOptions(OPTIONS, 'deploy').map((option) => option.value)).toEqual(['role-3']);
    expect(matchOptions(OPTIONS, 'role 7').map((option) => option.value)).toEqual(['role-7']);
    expect(matchOptions(OPTIONS, '  ')).toHaveLength(8);
  });

  it('adds with a click or Enter, removes with a chip button and limits the list', async () => {
    renderWithApp(<Harness />);
    expect(screen.getByText('None chosen yet')).toBeInTheDocument();
    expect(screen.getAllByRole('option')).toHaveLength(5);
    expect(screen.getByRole('status')).toHaveTextContent('3 more options match');
    await userEvent.click(screen.getByRole('option', { name: /role-1/ }));
    const search = screen.getByRole('combobox', { name: 'Find Roles' });
    await userEvent.type(search, 'deployers{Enter}');
    const chosen = screen.getByRole('list', { name: 'Chosen Roles' });
    expect(chosen).toHaveTextContent('role-1');
    expect(chosen).toHaveTextContent('role-3');
    expect(screen.getByRole('option', { name: /role-3/ })).toHaveAttribute('aria-selected', 'true');
    await userEvent.click(screen.getByRole('button', { name: 'Remove role-1' }));
    expect(chosen).not.toHaveTextContent('role-1');
  });

  it('shows chips without controls when disabled', () => {
    renderWithApp(
      <ChipPicker label="Roles" value={['role-1']} onChange={() => {}} options={OPTIONS} disabled />,
    );
    expect(screen.getByText('role-1')).toBeInTheDocument();
    expect(screen.queryByRole('combobox')).toBeNull();
    expect(screen.queryByRole('button')).toBeNull();
  });
});
