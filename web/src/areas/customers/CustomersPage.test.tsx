import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { CUSTOMERS, type Customer } from './api';
import CustomersPage from './CustomersPage';

afterEach(() => {
  vi.unstubAllGlobals();
});

const DAY_MS = 24 * 60 * 60 * 1000;

function customer(patch: Partial<Customer>): Customer {
  return {
    id: 'acme',
    name: 'Acme',
    reference: 'CT-1',
    note: '',
    disabled: false,
    createdAt: '2026-10-01T10:00:00Z',
    createdBy: 'admin',
    tokens: 1,
    nearestExpiry: null,
    downloads30d: 1200,
    bytes30d: 3_500_000,
    lastAccessAt: null,
    ...patch,
  };
}

describe('the customers list', () => {
  it('renders the rows and colours the nearest expiry by how close it is', async () => {
    const soon = customer({
      id: 'acme',
      name: 'Acme',
      nearestExpiry: new Date(Date.now() + 10 * DAY_MS).toISOString(),
    });
    const past = customer({
      id: 'globex',
      name: 'Globex',
      disabled: true,
      nearestExpiry: new Date(Date.now() - 2 * DAY_MS).toISOString(),
    });
    const fake = fakeServer({ [`GET ${CUSTOMERS}`]: () => jsonResponse([soon, past]) });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<CustomersPage />);
    expect(await screen.findByRole('link', { name: /Acme/ })).toHaveAttribute(
      'href',
      '/admin/customers/acme',
    );
    expect(screen.getByRole('link', { name: /Globex/ })).toHaveTextContent('Disabled');
    expect(document.querySelectorAll('[data-accent-color="amber"]')).toHaveLength(1);
    expect(document.querySelectorAll('[data-accent-color="red"]')).toHaveLength(1);
  });

  it('creates a customer with the four fields and shows the server field error on the id', async () => {
    let attempts = 0;
    const created = customer({ id: 'globex', name: 'Globex Ltd', reference: 'CT-7', note: 'Pilot' });
    const fake = fakeServer({
      [`GET ${CUSTOMERS}`]: () => jsonResponse([customer({})]),
      [`POST ${CUSTOMERS}`]: () => {
        attempts += 1;
        if (attempts === 1) {
          return jsonResponse([{ id: 'id', message: 'choose another id' }], 400);
        }
        return jsonResponse(created, 201);
      },
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<CustomersPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Create Customer' }));
    const dialog = await screen.findByRole('dialog', { name: 'Create Customer' });
    await userEvent.type(screen.getByLabelText(/^ID/), 'globex');
    await userEvent.type(screen.getByLabelText(/^Name/), 'Globex Ltd');
    await userEvent.type(screen.getByLabelText('Reference'), 'CT-7');
    await userEvent.type(screen.getByLabelText('Note'), 'Pilot');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('choose another id')).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    const posts = fake.calls.filter((call) => call.method === 'POST');
    expect(posts).toHaveLength(2);
    expect(posts[1]?.body).toEqual({ id: 'globex', name: 'Globex Ltd', reference: 'CT-7', note: 'Pilot' });
  });
});
