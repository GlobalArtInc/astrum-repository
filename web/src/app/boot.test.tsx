import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import INDEX from '../../index.html?raw';
import THEME from '../../public/theme.js?raw';
import { renderWithApp } from '../test/render';
import { setMedia } from '../test/setup';
import { Starting } from './Startup';

function themeAfterBoot(stored: string | null): string | undefined {
  if (stored !== null) {
    window.localStorage.setItem('astrum-theme-preference', stored);
  }
  document.documentElement.dataset.theme = 'unset';
  new Function(THEME)();
  return document.documentElement.dataset.theme;
}

afterEach(() => {
  delete document.documentElement.dataset.theme;
});

describe('the first paint', () => {
  it('takes the stored theme before the app loads', () => {
    expect(themeAfterBoot(null)).toBe('dark');
    expect(themeAfterBoot('light')).toBe('light');
    expect(themeAfterBoot('dark')).toBe('dark');
    expect(themeAfterBoot('system')).toBe('dark');
    setMedia('(prefers-color-scheme: light)', true);
    expect(themeAfterBoot('system')).toBe('light');
  });

  it('shows the loading screen from the page itself', () => {
    const page = new DOMParser().parseFromString(INDEX, 'text/html');
    const boot = page.querySelector('#root > .boot');
    expect(boot?.getAttribute('role')).toBe('status');
    expect(boot?.querySelector('.boot-logo svg')).not.toBeNull();
    expect(boot?.querySelector('.boot-status')?.textContent).toBe('Loading ...');
    const scripts = [...page.querySelectorAll('script')].map((script) => script.getAttribute('src'));
    expect(scripts).toEqual(['/theme.js', '/src/main.tsx']);
    expect(page.head.querySelector('script')?.getAttribute('type')).toBeNull();
  });

  it('keeps the same screen while the app starts', () => {
    renderWithApp(<Starting />);
    const status = screen.getByRole('status', { name: 'Loading Astrum Repository' });
    expect(status).toHaveClass('boot');
    expect(status).toHaveTextContent('Loading ...');
    expect(status.querySelector('.boot-logo svg')).not.toBeNull();
  });
});
