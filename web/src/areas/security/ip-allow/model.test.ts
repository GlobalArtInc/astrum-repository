import { describe, expect, it } from 'vitest';
import { ApiError, type FieldError } from '../../../api';
import { entryProblem, isLockout, localizeLockout } from './model';

describe('allow list entries', () => {
  it('accepts addresses and networks and names the entries the server refuses', () => {
    expect(entryProblem('10.1.1.1')).toBeUndefined();
    expect(entryProblem(' 10.0.0.0/8 ')).toBeUndefined();
    expect(entryProblem('fd00::/8')).toBeUndefined();
    expect(entryProblem('10.0.0')).toBe('malformed');
    expect(entryProblem('10.0.0.0/33')).toBe('malformed');
    expect(entryProblem('0.0.0.0/0')).toBe('wholeFamily');
    expect(entryProblem('::/0')).toBe('wholeFamily');
  });
});

describe('server refusals of the allow list', () => {
  const lockout: FieldError = {
    id: 'management',
    message:
      'does not contain your address 10.1.1.1; saving it would lock you out of the management interface',
  };

  it('recognises the lockout refusal on the management list only', () => {
    expect(isLockout(lockout)).toBe(true);
    expect(
      isLockout({ id: 'management', message: '"x" is neither an IP address nor a range such as 10.0.0.0/8' }),
    ).toBe(false);
    expect(isLockout({ ...lockout, id: 'content' })).toBe(false);
  });

  it('replaces the lockout message with the given text and keeps the other problems', () => {
    const other: FieldError = { id: 'content', message: 'has 300 entries; at most 256 are allowed' };
    const failure = new ApiError(400, 'bad', { fieldErrors: [lockout, other] });
    const localized = localizeLockout(failure, 'Add your address') as ApiError;
    expect(localized).toBeInstanceOf(ApiError);
    expect(localized.status).toBe(400);
    expect(localized.fieldErrors).toEqual([{ id: 'management', message: 'Add your address' }, other]);
    const network = new Error('offline');
    expect(localizeLockout(network, 'Add your address')).toBe(network);
  });
});
