

/**
 * Form utilities for building XState-powered forms with Radix UI components.
 *
 * @example
 * ```tsx
 * import { createFormMachine, useForm } from '@globalart/astrum-ui-plugin/interface/form';
 *
 * const userFormMachine = createFormMachine({
 *   id: 'user-form',
 *   context: { data: { name: '', email: '' } },
 *   actions: {
 *     validate: (ctx) => ({
 *       name: !ctx.data.name ? 'Required' : null,
 *     }),
 *   },
 *   services: {
 *     save: async (ctx) => api.saveUser(ctx.data),
 *   },
 * });
 *
 * function UserForm() {
 *   const form = useForm(userFormMachine, { formId: 'user-form' });
 *
 *   return (
 *     <form onSubmit={(e) => { e.preventDefault(); form.submit(); }}>
 *       <input {...form.field('name')} />
 *       <button type="submit">Save</button>
 *     </form>
 *   );
 * }
 * ```
 */

// Machine factory
export { createFormMachine } from './createFormMachine';
export type { FormMachineConfig } from './createFormMachine';

// React hook
export { useForm } from './useForm';

// Types
export type {
  FormContext,
  FormEvent,
  ValidationErrors,
  ValidateFn,
  FieldProps,
  CheckboxProps,
  SelectProps,
  UseFormReturn,
} from './types';

// Utilities
export { hasValidationErrors, extractErrorMessage, toPathArray } from './utils';
