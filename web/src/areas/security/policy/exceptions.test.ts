import { describe, expect, it } from 'vitest';
import {
  EMPTY_EXCEPTION_FORM,
  type ExceptionForm,
  expired,
  expiresAtOf,
  type PolicyException,
  requestOf,
  subjectsOf,
} from './exceptions';
import { DAY } from './model';

const NOW = Date.parse('2026-11-11T10:00:00Z');

describe('policy exceptions', () => {
  it('sends only the parts of a component that are filled', () => {
    const form: ExceptionForm = {
      ...EMPTY_EXCEPTION_FORM,
      rule: 'vulnerability',
      subject: 'component',
      format: 'npm',
      name: 'lodash',
      version: ' 4.17.20 ',
      reason: ' pinned by the platform team ',
      expiry: 'never',
    };
    expect(requestOf(form, NOW)).toEqual({
      rule: 'vulnerability',
      format: 'npm',
      name: 'lodash',
      version: '4.17.20',
      reason: 'pinned by the platform team',
      expiresAt: null,
    });
  });

  it('sends an advisory with the component it is excepted for', () => {
    const form: ExceptionForm = {
      ...EMPTY_EXCEPTION_FORM,
      rule: 'vulnerability',
      subject: 'advisory',
      advisory: ' GHSA-aaaa-bbbb-cccc ',
      policy: 'strict',
      format: 'maven2',
      namespace: 'org.acme',
      name: 'lib',
      version: '1.0',
      reason: 'vendor patch in review',
      expiry: 'never',
    };
    expect(requestOf(form, NOW)).toEqual({
      policy: 'strict',
      rule: 'vulnerability',
      advisory: 'GHSA-aaaa-bbbb-cccc',
      format: 'maven2',
      namespace: 'org.acme',
      name: 'lib',
      version: '1.0',
      reason: 'vendor patch in review',
      expiresAt: null,
    });
    expect(requestOf({ ...form, format: '', namespace: '', name: '', version: '' }, NOW)).toEqual({
      policy: 'strict',
      rule: 'vulnerability',
      advisory: 'GHSA-aaaa-bbbb-cccc',
      reason: 'vendor patch in review',
      expiresAt: null,
    });
  });

  it('counts the presets from now and offers only the subjects a rule can take', () => {
    expect(Date.parse(expiresAtOf({ ...EMPTY_EXCEPTION_FORM, expiry: '7d' }, NOW) ?? '') - NOW).toBe(7 * DAY);
    expect(subjectsOf('license')).toEqual(['license']);
    expect(subjectsOf('signature')).not.toContain('advisory');
  });

  it('marks an exception expired once its end has passed', () => {
    const exception: PolicyException = {
      id: '1',
      policy: null,
      rule: 'license',
      advisory: null,
      license: 'MIT',
      format: null,
      namespace: null,
      name: null,
      version: null,
      digest: null,
      reason: 'approved',
      expiresAt: '2026-11-01T00:00:00Z',
      createdBy: 'admin',
      createdAt: '2026-10-01T00:00:00Z',
    };
    expect(expired(exception, NOW)).toBe(true);
    expect(expired({ ...exception, expiresAt: null }, NOW)).toBe(false);
  });
});
