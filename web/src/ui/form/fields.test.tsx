import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderWithApp } from '../../test/render';
import {
  CheckboxField,
  NumberField,
  parseNumber,
  SecretField,
  SwitchField,
  TextAreaField,
  TextField,
} from './fields';
import { filterOptions, MultiSelectField } from './MultiSelectField';

describe('form fields', () => {
  it('labels inputs and shows hints and errors', async () => {
    const change = vi.fn();
    renderWithApp(
      <TextField label="Name" value="" onChange={change} hint="Letters only" error="Required" required />,
    );
    const input = screen.getByRole('textbox', { name: /Name/ });
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAccessibleDescription('Letters only Required');
    await userEvent.type(input, 'a');
    expect(change).toHaveBeenCalledWith('a');
  });

  it('reads numbers or nothing', async () => {
    expect(parseNumber('')).toBeNull();
    expect(parseNumber('12')).toBe(12);
    expect(parseNumber('x')).toBeNull();
    const change = vi.fn();
    renderWithApp(<NumberField label="Port" value={null} onChange={change} />);
    await userEvent.type(screen.getByRole('spinbutton', { name: 'Port' }), '7');
    expect(change).toHaveBeenLastCalledWith(7);
  });

  it('keeps a stored secret unless a new one is typed', async () => {
    renderWithApp(<SecretField label="Password" value="" onChange={vi.fn()} stored />);
    const input = screen.getByLabelText('Password');
    expect(input).toHaveAttribute('type', 'password');
    expect(input).not.toBeRequired();
    expect(screen.getByText('A value is stored. Leave the field empty to keep it.')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Show' }));
    expect(input).toHaveAttribute('type', 'text');
    expect(screen.getByRole('button', { name: 'Hide' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('toggles checkboxes and switches', async () => {
    const checked = vi.fn();
    const switched = vi.fn();
    renderWithApp(
      <>
        <CheckboxField label="Online" value={false} onChange={checked} description="Serve requests" />
        <SwitchField label="Strict" value={true} onChange={switched} />
      </>,
    );
    await userEvent.click(screen.getByRole('checkbox', { name: /Online/ }));
    expect(checked).toHaveBeenCalledWith(true);
    await userEvent.click(screen.getByRole('switch', { name: 'Strict' }));
    expect(switched).toHaveBeenCalledWith(false);
  });

  it('edits long texts', async () => {
    const change = vi.fn();
    renderWithApp(<TextAreaField label="Expression" value="" onChange={change} monospace />);
    await userEvent.type(screen.getByRole('textbox', { name: 'Expression' }), 'x');
    expect(change).toHaveBeenCalledWith('x');
  });
});

function Roles() {
  const [value, setValue] = useState<string[]>(['admin']);
  return (
    <MultiSelectField
      label="Roles"
      value={value}
      onChange={setValue}
      options={[
        { value: 'admin', label: 'Administrators' },
        { value: 'dev', label: 'Developers' },
        { value: 'ops', label: 'Operators' },
      ]}
    />
  );
}

describe('the multi-select', () => {
  it('filters options', () => {
    const options = [
      { value: 'a', label: 'Alpha' },
      { value: 'b', label: 'Beta' },
    ];
    expect(filterOptions(options, ' al')).toEqual([options[0]]);
    expect(filterOptions(options, 'b')).toEqual([options[1]]);
    expect(filterOptions(options, '')).toEqual(options);
  });

  it('chooses with the keyboard and removes chips', async () => {
    renderWithApp(<Roles />);
    expect(screen.getByRole('button', { name: 'Remove Administrators' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: /Roles/ }));
    const filter = await screen.findByRole('combobox', { name: 'Filter' });
    await userEvent.type(filter, 'opera');
    expect(screen.getAllByRole('option')).toHaveLength(1);
    await userEvent.keyboard('{Enter}');
    expect(screen.getByRole('option', { name: 'Operators' })).toHaveAttribute('aria-selected', 'true');
    await userEvent.keyboard('{Escape}');
    expect(screen.getByRole('button', { name: 'Roles' })).toHaveTextContent('2 chosen');
    await userEvent.click(screen.getByRole('button', { name: 'Remove Administrators' }));
    expect(screen.getByRole('button', { name: 'Roles' })).toHaveTextContent('1 chosen');
  });
});
