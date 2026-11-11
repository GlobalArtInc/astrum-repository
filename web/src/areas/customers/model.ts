import type { Translate } from '../../i18n';
import type { Errors } from '../../ui';

export const READ = 'astrum:customers:read';
export const CREATE = 'astrum:customers:create';
export const UPDATE = 'astrum:customers:update';
export const DELETE = 'astrum:customers:delete';

export const EXPIRY_WARNING_DAYS = 30;
export const NAME_MAX = 200;
export const REFERENCE_MAX = 200;
export const NOTE_MAX = 2000;

const ID_PATTERN = /^[a-z0-9-]{1,64}$/;
const DAY_MS = 24 * 60 * 60 * 1000;

export type ExpiryTone = 'warning' | 'danger';

export interface CustomerValues {
  name: string;
  reference: string;
  note: string;
  disabled: boolean;
}

export interface NewCustomerValues extends CustomerValues {
  id: string;
}

export interface CustomerRequest {
  name: string;
  reference: string;
  note: string;
  disabled: boolean;
}

export interface NewCustomerRequest {
  id: string;
  name: string;
  reference: string;
  note: string;
}

export const EMPTY_CUSTOMER: NewCustomerValues = {
  id: '',
  name: '',
  reference: '',
  note: '',
  disabled: false,
};

export function fromCustomer(customer: CustomerValues): CustomerValues {
  return {
    name: customer.name,
    reference: customer.reference,
    note: customer.note,
    disabled: customer.disabled,
  };
}

export function idProblem(t: Translate, id: string): string | undefined {
  return ID_PATTERN.test(id) ? undefined : t('customers.errors.id');
}

export function nameProblem(t: Translate, name: string): string | undefined {
  return name.trim() === '' ? t('customers.errors.name') : undefined;
}

export function validateCustomer(t: Translate, values: CustomerValues): Errors<CustomerValues> {
  const name = nameProblem(t, values.name);
  return name ? { name } : {};
}

export function validateNewCustomer(t: Translate, values: NewCustomerValues): Errors<NewCustomerValues> {
  const errors: Errors<NewCustomerValues> = validateCustomer(t, values);
  const id = idProblem(t, values.id);
  if (id) {
    errors.id = id;
  }
  return errors;
}

export function toCustomerRequest(values: CustomerValues): CustomerRequest {
  return {
    name: values.name.trim(),
    reference: values.reference.trim(),
    note: values.note.trim(),
    disabled: values.disabled,
  };
}

export function toNewCustomerRequest(values: NewCustomerValues): NewCustomerRequest {
  return {
    id: values.id,
    name: values.name.trim(),
    reference: values.reference.trim(),
    note: values.note.trim(),
  };
}

export function expiryTone(expiry: string | null, now: number): ExpiryTone | undefined {
  if (expiry === null) {
    return undefined;
  }
  const at = Date.parse(expiry);
  if (at <= now) {
    return 'danger';
  }
  return at - now <= EXPIRY_WARNING_DAYS * DAY_MS ? 'warning' : undefined;
}
