import {
  getEffectiveTaskStatus,
  getTaskScheduleLabel,
  getTaskStatusBadgeColor,
  getTaskStatusLabel,
} from '../taskLabels';

describe('taskLabels', () => {
  it('names every state the server reports, including a disabled task', () => {
    expect(getTaskStatusLabel('WAITING')).toBe('Waiting');
    expect(getTaskStatusLabel('RUNNING')).toBe('Running');
    expect(getTaskStatusLabel('INTERRUPTED')).toBe('Interrupted');
    expect(getTaskStatusLabel('DISABLED')).toBe('Disabled');
    expect(getTaskStatusLabel('SOMETHING_NEW')).toBe('SOMETHING_NEW');
  });

  it('names the schedules', () => {
    expect(getTaskScheduleLabel('manual')).toBe('Manual');
    expect(getTaskScheduleLabel('daily')).toBe('Daily');
    expect(getTaskScheduleLabel('advanced')).toBe('Cron');
    expect(getTaskScheduleLabel(null)).toBe('-');
  });

  it('shows a disabled task as disabled whatever its scheduler state', () => {
    expect(getEffectiveTaskStatus({ enabled: false, status: 'WAITING' })).toBe('DISABLED');
    expect(getEffectiveTaskStatus({ enabled: true, status: 'RUNNING' })).toBe('RUNNING');
  });

  it('colours the states', () => {
    expect(getTaskStatusBadgeColor('RUNNING')).toBe('blue');
    expect(getTaskStatusBadgeColor('OK')).toBe('green');
    expect(getTaskStatusBadgeColor('FAILED')).toBe('red');
    expect(getTaskStatusBadgeColor('DISABLED')).toBe('gray');
  });
});
