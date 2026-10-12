import { describe, expect, it } from 'vitest';
import { fingerprint, onboardingTarget, sessionMarker, shouldOffer } from './watcher';

const BASE = {
  signedIn: true,
  administrator: true,
  passwordFile: '/data/admin.password',
  pathname: '/',
  offered: null,
  marker: 'm1',
};

describe('the onboarding offer', () => {
  it('is made once per session to administrators while the password file exists', () => {
    expect(shouldOffer(BASE)).toBe(true);
    expect(shouldOffer({ ...BASE, offered: 'm1' })).toBe(false);
    expect(shouldOffer({ ...BASE, offered: 'm0' })).toBe(true);
    expect(shouldOffer({ ...BASE, passwordFile: null })).toBe(false);
    expect(shouldOffer({ ...BASE, administrator: false })).toBe(false);
    expect(shouldOffer({ ...BASE, signedIn: false })).toBe(false);
    expect(shouldOffer({ ...BASE, pathname: '/onboarding' })).toBe(false);
  });

  it('ties the session to the anti-forgery cookie without storing it', () => {
    const first = sessionMarker('admin', 'astrum_csrf=one; astrum-language=en');
    expect(first).toBe(sessionMarker('admin', 'astrum_csrf=one'));
    expect(first).not.toBe(sessionMarker('admin', 'astrum_csrf=two'));
    expect(first).not.toContain('one');
    expect(fingerprint('abc')).toMatch(/^[0-9a-f]{8}$/);
  });

  it('remembers where the user was going', () => {
    expect(onboardingTarget('/')).toBe('/onboarding');
    expect(onboardingTarget('/admin/security/users?q=a')).toBe(
      '/onboarding?returnTo=%2Fadmin%2Fsecurity%2Fusers%3Fq%3Da',
    );
  });
});

describe('the onboarding area', () => {
  it('registers its watcher for the shell', async () => {
    const { buildRegistry } = await import('../../app/registry');
    const { onboardingArea } = await import('.');
    const { OnboardingWatcher } = await import('./watcher');
    expect(buildRegistry([onboardingArea]).watchers).toEqual([OnboardingWatcher]);
  });
});
