import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import CapabilitiesPage from './CapabilitiesPage';
import CreateCapabilityPage from './CreateCapabilityPage';
import { CAPABILITIES, TYPES } from './model';

afterEach(() => {
  vi.unstubAllGlobals();
});

const field = (
  id: string,
  type: string,
  label: string,
  required = true,
  initialValue: string | null = null,
) => ({
  id,
  type,
  label,
  helpText: null,
  required,
  initialValue,
  minValue: null,
  maxValue: null,
});

const TYPE_LIST = [
  {
    id: 'baseurl',
    name: 'Base URL',
    about: 'Sets the address of the server.',
    formFields: [field('url', 'url', 'URL')],
  },
  {
    id: 'rapture.settings',
    name: 'UI: Settings',
    about: 'Adjusts the user interface.',
    formFields: [field('title', 'string', 'Title', true, 'GlobalArt Astrum Repository')],
  },
];

const capability = (type: string, typeName: string, category: string, enabled: boolean) => ({
  id: type,
  type,
  typeName,
  enabled,
  active: enabled,
  error: false,
  state: enabled ? 'active' : 'disabled',
  notes: '',
  description: null,
  status: null,
  properties: {},
  tags: { Category: category },
});

describe('the capability pages', () => {
  it('filter the list by state and category', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({
        [`GET ${CAPABILITIES}`]: () =>
          jsonResponse([
            capability('rapture.settings', 'UI: Settings', 'UI', true),
            capability('baseurl', 'Base URL', 'Core', false),
          ]),
      }).fetch,
    );
    renderWithApp(<CapabilitiesPage />);
    await screen.findByText('UI: Settings');
    const table = screen.getByRole('table');
    expect(within(table).getAllByRole('row')).toHaveLength(3);
    expect(screen.getByText('Shown 2 of 2')).toBeInTheDocument();
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('checkbox', { name: /Active/ }));
    expect(within(table).getAllByRole('row')).toHaveLength(2);
    expect(within(table).getByText('UI: Settings')).toBeInTheDocument();
    expect(screen.getByText('Shown 1 of 2')).toBeInTheDocument();
  });

  it('create a capability in two steps', async () => {
    const fake = fakeServer({
      [`GET ${TYPES}`]: () => jsonResponse(TYPE_LIST),
      [`GET ${CAPABILITIES}`]: () =>
        jsonResponse([capability('rapture.settings', 'UI: Settings', 'UI', true)]),
      [`POST ${CAPABILITIES}`]: () => jsonResponse(capability('baseurl', 'Base URL', 'Core', true), 201),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<CreateCapabilityPage />);
    expect(await screen.findByText('Types available: 1 of 2')).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: /UI: Settings/ })).toBeDisabled();
    await userEvent.click(screen.getByRole('radio', { name: /Base URL/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(await screen.findByRole('heading', { name: 'Configure' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('Fill in “URL”')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^URL/), 'https://repo.example.com');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'POST')?.body).toEqual({
      type: 'baseurl',
      enabled: true,
      notes: '',
      properties: { url: 'https://repo.example.com' },
    });
  }, 30_000);
});
