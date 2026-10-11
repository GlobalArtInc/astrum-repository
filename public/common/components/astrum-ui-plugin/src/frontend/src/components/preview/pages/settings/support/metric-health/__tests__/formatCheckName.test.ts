import { formatCheckName } from '../types';

describe('formatCheckName', () => {
  it('keeps a name that is already readable', () => {
    expect(formatCheckName('Available CPUs')).toBe('Available CPUs');
    expect(formatCheckName('Re-encryption required')).toBe('Re-encryption required');
  });

  it('splits a camel case name into words and drops the check suffix', () => {
    expect(formatCheckName('threadDeadlockHealthCheck')).toBe('Thread Deadlock');
    expect(formatCheckName('DefaultRoleRealm')).toBe('Default Role Realm');
    expect(formatCheckName('Scheduler')).toBe('Scheduler');
  });

  it('keeps acronyms together', () => {
    expect(formatCheckName('HTTPProxyCheck')).toBe('HTTP Proxy');
  });

  it('uses the simple name of a class name', () => {
    expect(formatCheckName('com.codahale.metrics.health.jvm.ThreadDeadlockHealthCheck')).toBe('Thread Deadlock');
  });
});
