export const STORAGE_KEY = 'astrum-docs-banner';
export const DISMISSED_CLASS = 'banner-dismissed';
export const REPOSITORY_URL = 'https://github.com/GlobalArtInc/astrum-repository';

export function dismiss(): void {
  document.documentElement.classList.add(DISMISSED_CLASS);
  try {
    window.localStorage.setItem(STORAGE_KEY, '1');
  } catch {
    return;
  }
}
