import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../api';
import { Providers, testClient } from '../../test/render';
import { mapServerErrors, useForm } from './useForm';

interface Values {
  name: string;
  port: number | null;
  online: boolean;
}

const INITIAL: Values = { name: 'releases', port: null, online: true };

function wrapper({ children }: { children: ReactNode }) {
  return <Providers client={testClient()}>{children}</Providers>;
}

describe('useForm', () => {
  it('tracks values and whether they changed', () => {
    const { result } = renderHook(() => useForm({ initial: INITIAL, onSubmit: vi.fn() }), { wrapper });
    expect(result.current.dirty).toBe(false);
    act(() => result.current.field('name').onChange('snapshots'));
    expect(result.current.values.name).toBe('snapshots');
    expect(result.current.dirty).toBe(true);
    act(() => result.current.set('name', 'releases'));
    expect(result.current.dirty).toBe(false);
  });

  it('stops at client validation', async () => {
    const submit = vi.fn();
    const { result } = renderHook(
      () =>
        useForm({
          initial: INITIAL,
          validate: (values) => (values.name ? {} : { name: 'Enter a name' }),
          onSubmit: submit,
        }),
      { wrapper },
    );
    act(() => result.current.set('name', ''));
    await act(() => result.current.submit());
    expect(submit).not.toHaveBeenCalled();
    expect(result.current.field('name').error).toBe('Enter a name');
    act(() => result.current.set('name', 'x'));
    expect(result.current.errors.name).toBeUndefined();
  });

  it('submits and takes the saved values as the new baseline', async () => {
    const submit = vi.fn();
    const { result } = renderHook(() => useForm({ initial: INITIAL, onSubmit: submit }), { wrapper });
    act(() => result.current.set('port', 8082));
    await act(() => result.current.submit());
    expect(submit).toHaveBeenCalledWith({ ...INITIAL, port: 8082 });
    expect(result.current.dirty).toBe(false);
  });

  it('puts the server field errors next to their fields', async () => {
    const failure = new ApiError(400, 'bad', {
      fieldErrors: [
        { id: 'PARAMETER name', message: 'is already used' },
        { id: 'docker.port', message: 'is taken' },
        { id: '*', message: 'Something else' },
      ],
    });
    const { result } = renderHook(
      () => useForm({ initial: INITIAL, onSubmit: () => Promise.reject(failure) }),
      { wrapper },
    );
    await act(() => result.current.submit());
    expect(result.current.errors).toEqual({ name: 'is already used', port: 'is taken' });
    expect(result.current.formError).toBe('Something else');
    expect(result.current.submitting).toBe(false);
  });

  it('describes other failures for the whole form', async () => {
    const { result } = renderHook(
      () => useForm({ initial: INITIAL, onSubmit: () => Promise.reject(new ApiError(403, 'no')) }),
      { wrapper },
    );
    await act(() => result.current.submit());
    expect(result.current.formError).toBe('You do not have the permission this needs.');
  });

  it('maps ids by exact name or by the last part', () => {
    const mapped = mapServerErrors<Values>(
      new ApiError(400, 'x', { fieldErrors: [{ id: 'storage.name', message: 'm' }] }),
      ['name'],
    );
    expect(mapped.errors).toEqual({ name: 'm' });
    expect(mapped.unmatched).toEqual([]);
  });
});
