import { ApiError } from '../../../api';
import type { Translate } from '../../../i18n';
import { describeError } from '../../../ui';

export function failureText(t: Translate, error: unknown): string {
  if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
    return error.fieldErrors.map((problem) => problem.message).join(' ');
  }
  return describeError(t, error);
}
