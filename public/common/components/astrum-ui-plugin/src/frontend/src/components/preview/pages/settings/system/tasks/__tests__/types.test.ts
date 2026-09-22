

import {
  TERMINAL_TASK_STATUSES,
  isTerminalStatus,
  isActiveStatus,
} from '../types';

// Single client-side source of truth for which task statuses are terminal vs
// active, reused by the polling gates on both the detail and list pages so the
// "keep polling / stop polling" decision is defined in exactly one place.
describe('task status classification', () => {
  it('defines the terminal status set as the DONE-group outcomes', () => {
    expect([...TERMINAL_TASK_STATUSES].sort()).toEqual(
      ['CANCELED', 'FAILED', 'INTERRUPTED', 'OK'],
    );
  });

  it('isTerminalStatus is true only for terminal outcomes', () => {
    expect(isTerminalStatus('OK')).toBe(true);
    expect(isTerminalStatus('FAILED')).toBe(true);
    expect(isTerminalStatus('CANCELED')).toBe(true);
    expect(isTerminalStatus('INTERRUPTED')).toBe(true);

    expect(isTerminalStatus('WAITING')).toBe(false);
    expect(isTerminalStatus('RUNNING')).toBe(false);
    expect(isTerminalStatus('BLOCKED')).toBe(false);
    expect(isTerminalStatus(null)).toBe(false);
    expect(isTerminalStatus(undefined)).toBe(false);
  });

  it('isActiveStatus is true while the task is doing work', () => {
    expect(isActiveStatus('RUNNING')).toBe(true);
    expect(isActiveStatus('BLOCKED')).toBe(true);

    expect(isActiveStatus('WAITING')).toBe(false);
    expect(isActiveStatus('OK')).toBe(false);
    expect(isActiveStatus('FAILED')).toBe(false);
    expect(isActiveStatus(null)).toBe(false);
    expect(isActiveStatus(undefined)).toBe(false);
  });
});
