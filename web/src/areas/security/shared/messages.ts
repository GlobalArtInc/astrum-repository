import { ApiError } from '../../../api';
import type { Translate } from '../../../i18n';

export function refusedBy(t: Translate, error: unknown): unknown {
  if (error instanceof ApiError && error.status === 403 && error.message) {
    return new Error(t('security.form.refused', { message: error.message }));
  }
  return error;
}

export async function explainRefusal<T>(t: Translate, work: () => Promise<T>): Promise<T> {
  try {
    return await work();
  } catch (error) {
    throw refusedBy(t, error);
  }
}
