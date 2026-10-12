import { type FormEvent, useCallback, useMemo, useRef, useState } from 'react';
import { ApiError, fieldName } from '../../api';
import { useT } from '../../i18n';
import { describeError } from '../ErrorState';

export type Errors<T> = Partial<Record<keyof T & string, string>>;

export interface FieldBinding<V> {
  name: string;
  value: V;
  onChange(value: V): void;
  error?: string;
}

export interface FormOptions<T> {
  initial: T;
  validate?: (values: T) => Errors<T>;
  onSubmit: (values: T) => void | Promise<void>;
}

export interface Form<T> {
  values: T;
  errors: Errors<T>;
  formError: string | null;
  dirty: boolean;
  submitting: boolean;
  set<K extends keyof T & string>(name: K, value: T[K]): void;
  field<K extends keyof T & string>(name: K): FieldBinding<T[K]>;
  submit(event?: FormEvent): Promise<void>;
  reset(values?: T): void;
  setErrors(errors: Errors<T>, formError?: string | null): void;
}

export function mapServerErrors<T>(
  error: ApiError,
  fields: readonly string[],
): { errors: Errors<T>; unmatched: string[] } {
  const errors: Record<string, string> = {};
  const unmatched: string[] = [];
  for (const problem of error.fieldErrors) {
    const id = fieldName(problem.id);
    const name =
      fields.find((field) => field === id) ?? fields.find((field) => id.split('.').pop() === field);
    if (name && !errors[name]) {
      errors[name] = problem.message;
    } else if (!name) {
      unmatched.push(problem.message);
    }
  }
  return { errors: errors as Errors<T>, unmatched };
}

function same(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function useForm<T extends object>({ initial, validate, onSubmit }: FormOptions<T>): Form<T> {
  const t = useT();
  const [baseline, setBaseline] = useState(initial);
  const [values, setValues] = useState(initial);
  const [errors, setFieldErrors] = useState<Errors<T>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const latest = useRef(values);
  latest.current = values;

  const set = useCallback(<K extends keyof T & string>(name: K, value: T[K]) => {
    setValues((current) => ({ ...current, [name]: value }));
    setFieldErrors((current) => {
      if (!(name in current)) {
        return current;
      }
      const next = { ...current };
      delete next[name];
      return next;
    });
  }, []);

  const reset = useCallback(
    (next?: T) => {
      const target = next ?? baseline;
      setBaseline(target);
      setValues(target);
      setFieldErrors({});
      setFormError(null);
    },
    [baseline],
  );

  const setErrors = useCallback((next: Errors<T>, message: string | null = null) => {
    setFieldErrors(next);
    setFormError(message);
  }, []);

  const submit = useCallback(
    async (event?: FormEvent) => {
      event?.preventDefault();
      const current = latest.current;
      const found = validate?.(current) ?? {};
      if (Object.keys(found).length > 0) {
        setErrors(found);
        return;
      }
      setSubmitting(true);
      setErrors({});
      try {
        await onSubmit(current);
        setBaseline(current);
      } catch (failure) {
        if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
          const mapped = mapServerErrors<T>(failure, Object.keys(current));
          setErrors(mapped.errors, mapped.unmatched.length > 0 ? mapped.unmatched.join(' ') : null);
        } else {
          setErrors({}, describeError(t, failure));
        }
      } finally {
        setSubmitting(false);
      }
    },
    [onSubmit, validate, setErrors, t],
  );

  const dirty = useMemo(() => !same(values, baseline), [values, baseline]);

  const field = <K extends keyof T & string>(name: K): FieldBinding<T[K]> => ({
    name,
    value: values[name],
    onChange: (value: T[K]) => set(name, value),
    error: errors[name],
  });

  return { values, errors, formError, dirty, submitting, set, field, submit, reset, setErrors };
}
