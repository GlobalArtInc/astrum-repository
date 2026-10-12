import {
  browsePathOf,
  componentName,
  formatAgo,
  formatBytes,
  formatDay,
  formatDays,
  formatNumber,
  formatWeekday,
  isWeekend,
  usageLinks,
} from '../usageFormat';
import { labelStep, niceMax } from '../DownloadsChart';
import { component } from './usageDashboard.fixtures';

describe('usageFormat', () => {
  afterEach(() => {
    delete (window as unknown as { AS_I18N?: unknown }).AS_I18N;
  });

  it('formats numbers, sizes and days in English by default', () => {
    expect(formatNumber(2418)).toBe('2,418');
    expect(formatNumber(2.345, 1)).toBe('2.3');
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(186.4 * 1024 ** 3)).toBe('186 GB');
    expect(formatBytes(21.7 * 1024 ** 3)).toBe('21.7 GB');
    expect(formatDays(30)).toBe('30 days');
    expect(formatDay('2026-10-11')).toBe('Oct 11');
    expect(formatWeekday('2026-10-11')).toBe('Sun 11');
  });

  it('formats in the language of the page', () => {
    (window as unknown as { AS_I18N?: unknown }).AS_I18N = { language: 'ru', dictionaries: {} };

    expect(formatNumber(2418)).toBe('2 418');
    expect(formatDays(30)).toBe('30 дней');
    expect(formatDays(1)).toBe('1 день');
    expect(formatAgo('2026-10-10T11:00:00Z', Date.parse('2026-10-11T12:00:00Z'))).toBe('вчера');
  });

  it('tells weekends apart in UTC', () => {
    expect(isWeekend('2026-10-10')).toBe(true);
    expect(isWeekend('2026-10-11')).toBe(true);
    expect(isWeekend('2026-10-12')).toBe(false);
  });

  it('says how long ago something happened', () => {
    const now = Date.parse('2026-10-11T12:00:00Z');
    expect(formatAgo('2026-10-11T11:59:30Z', now)).toBe('now');
    expect(formatAgo('2026-10-11T11:48:00Z', now)).toBe('12 minutes ago');
    expect(formatAgo('2026-10-11T09:00:00Z', now)).toBe('3 hours ago');
    expect(formatAgo('2026-10-10T09:00:00Z', now)).toBe('yesterday');
    expect(formatAgo('2026-09-01T09:00:00Z', now)).toBe('Sep 1, 2026');
    expect(formatAgo('not a date', now)).toBe('');
  });

  it('writes component names the way their format does', () => {
    expect(componentName(component())).toBe('org.example:demo');
    expect(componentName(component({ format: 'npm', group: 'types', name: 'node' }))).toBe('@types/node');
    expect(componentName(component({ format: 'raw', group: null, name: 'a/b.txt' }))).toBe('a/b.txt');
    expect(componentName(component({ format: 'raw', group: '/tools', name: '/tools/keel.tgz' }))).toBe('tools/keel.tgz');
    expect(componentName(component({ format: 'pypi', group: 'x', name: 'y' }))).toBe('x/y');
  });

  it('builds the browse tree path when the server does not know it', () => {
    expect(browsePathOf(component())).toBe('org/example/demo/1.0.0');
    expect(browsePathOf(component({ browsePath: null }))).toBe('org/example/demo/1.0.0');
    expect(browsePathOf(component({ browsePath: null, format: 'npm', group: 'types', name: 'node' })))
      .toBe('@types/node/1.0.0');
    expect(browsePathOf(component({ browsePath: null, format: 'docker', group: null, name: 'app' })))
      .toBe('v2/app/tags/1.0.0');
  });

  it('links into the pages of Astrum One', () => {
    const odd = component({ repository: 'my repo', browsePath: 'a b/c#d' });
    expect(usageLinks.component(odd)).toBe('#browse/my%20repo/a%20b/c%23d');
    expect(usageLinks.repository('my repo')).toBe('#browse/my%20repo/');
    expect(usageLinks.search()).toBe('#browse/search');
    expect(usageLinks.cleanupPolicies()).toBe('#admin/repository/cleanup-policies');
  });

  it('rounds the top of the chart and spaces its labels', () => {
    expect(niceMax(0)).toBe(4);
    expect(niceMax(1)).toBe(2);
    expect(niceMax(7)).toBe(8);
    expect(niceMax(9)).toBe(10);
    expect(niceMax(108)).toBe(120);
    expect(niceMax(200)).toBe(240);
    expect(niceMax(1480)).toBe(2000);
    expect(labelStep(7)).toBe(1);
    expect(labelStep(30)).toBe(5);
    expect(labelStep(90)).toBe(15);
  });
});
