import { ApiError, type FieldError, fieldName } from '../../../api';
import { validNetwork } from '../ssrf/model';

export type EntryProblem = 'malformed' | 'wholeFamily';

const MANAGEMENT = 'management';
const LOCKOUT_MESSAGES = [
  'does not contain your address',
  'cannot be checked because your address is unknown',
];

export function entryProblem(text: string): EntryProblem | undefined {
  const value = text.trim();
  if (!validNetwork(value)) {
    return 'malformed';
  }
  return value.endsWith('/0') ? 'wholeFamily' : undefined;
}

export function isLockout(problem: FieldError): boolean {
  return (
    fieldName(problem.id) === MANAGEMENT && LOCKOUT_MESSAGES.some((text) => problem.message.startsWith(text))
  );
}

export function localizeLockout(failure: unknown, message: string): unknown {
  if (!(failure instanceof ApiError)) {
    return failure;
  }
  const fieldErrors: FieldError[] = failure.fieldErrors.map((problem) =>
    isLockout(problem) ? { id: problem.id, message } : problem,
  );
  return new ApiError(failure.status, failure.message, { fieldErrors });
}
