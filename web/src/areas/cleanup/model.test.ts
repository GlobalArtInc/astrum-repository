import { describe, expect, it } from 'vitest';
import {
  ANY_RELEASE,
  available,
  csvHref,
  EMPTY_POLICY,
  type FormatCriteria,
  formatLabel,
  fromPolicy,
  hasCriteria,
  matchesFilter,
  type Policy,
  type PolicyValues,
  toRequest,
  validate,
} from './model';

const FORMATS: FormatCriteria[] = [
  { id: '*', name: 'All Formats', availableCriteria: ['lastDownloaded', 'lastBlobUpdated'] },
  { id: 'raw', name: 'Raw', availableCriteria: ['regex', 'lastDownloaded', 'lastBlobUpdated'] },
  {
    id: 'maven2',
    name: 'Maven',
    availableCriteria: ['regex', 'isPrerelease', 'lastDownloaded', 'lastBlobUpdated'],
  },
];

const TEXTS = {
  name: 'name',
  nameRule: 'nameRule',
  format: 'format',
  notes: 'notes',
  criteria: 'criteria',
  age: 'age',
  usage: 'usage',
  days: 'days',
  regex: 'regex',
  regexInvalid: 'regexInvalid',
};

function values(change: Partial<PolicyValues>): PolicyValues {
  return { ...EMPTY_POLICY, ...change };
}

describe('cleanup policy values', () => {
  it('round-trips a stored policy', () => {
    const policy: Policy = {
      name: 'old-snapshots',
      format: 'maven2',
      notes: 'Snapshots',
      criteriaLastBlobUpdated: 30,
      criteriaLastDownloaded: null,
      criteriaReleaseType: 'PRERELEASES',
      criteriaAssetRegex: '.*-SNAPSHOT.*',
      inUseCount: 0,
      repositories: [],
    };
    const loaded = fromPolicy(policy);
    expect(loaded).toMatchObject({ ageOn: true, age: '30', usageOn: false, regexOn: true });
    expect(toRequest(loaded, available(FORMATS, 'maven2'))).toEqual({
      name: 'old-snapshots',
      format: 'maven2',
      notes: 'Snapshots',
      criteriaLastBlobUpdated: 30,
      criteriaLastDownloaded: null,
      criteriaReleaseType: 'PRERELEASES',
      criteriaAssetRegex: '.*-SNAPSHOT.*',
    });
  });

  it('drops criteria the format does not offer and a release type of either kind', () => {
    const chosen = values({
      name: 'any',
      format: '*',
      regexOn: true,
      regex: 'x',
      releaseType: 'RELEASES',
      usageOn: true,
      usage: '7',
    });
    expect(toRequest(chosen, available(FORMATS, '*'))).toMatchObject({
      criteriaAssetRegex: null,
      criteriaReleaseType: null,
      criteriaLastDownloaded: 7,
    });
    expect(
      toRequest(values({ format: 'maven2', releaseType: ANY_RELEASE }), available(FORMATS, 'maven2'))
        .criteriaReleaseType,
    ).toBeNull();
  });

  it('needs a day, usage or name criterion; the release type alone is not one', () => {
    const maven = available(FORMATS, 'maven2');
    expect(hasCriteria(values({ format: 'maven2', releaseType: 'RELEASES' }), maven)).toBe(false);
    expect(hasCriteria(values({ format: 'maven2', ageOn: true, age: '3' }), maven)).toBe(true);
    expect(
      validate(values({ name: 'p', format: 'maven2', releaseType: 'RELEASES' }), maven, true, TEXTS),
    ).toEqual({
      criteria: 'criteria',
    });
  });

  it('checks names, days and patterns like the server', () => {
    const raw = available(FORMATS, 'raw');
    expect(validate(values({ format: 'raw' }), raw, true, TEXTS).name).toBe('name');
    expect(validate(values({ name: 'bad name', format: 'raw' }), raw, true, TEXTS).name).toBe('nameRule');
    expect(validate(values({ name: 'ok', format: 'raw', ageOn: true }), raw, true, TEXTS).age).toBe('age');
    expect(validate(values({ name: 'ok', format: 'raw', ageOn: true, age: '0' }), raw, true, TEXTS).age).toBe(
      'days',
    );
    expect(
      validate(values({ name: 'ok', format: 'raw', usageOn: true, usage: '24856' }), raw, true, TEXTS).usage,
    ).toBe('days');
    expect(
      validate(values({ name: 'ok', format: 'raw', regexOn: true, regex: '(' }), raw, true, TEXTS).regex,
    ).toBe('regexInvalid');
    expect(
      validate(values({ name: 'ok', format: 'raw', usageOn: true, usage: '90' }), raw, true, TEXTS),
    ).toEqual({});
    expect(
      validate(values({ name: '', format: 'raw', usageOn: true, usage: '90' }), raw, false, TEXTS),
    ).toEqual({});
  });

  it('builds the CSV address from the criteria in the form', () => {
    const href = csvHref(
      values({ name: 'n', format: 'raw', regexOn: true, regex: 'docs/.*', usageOn: true, usage: '5' }),
      available(FORMATS, 'raw'),
      'raw-docs',
    );
    const url = new URL(href, 'http://localhost');
    expect(url.pathname).toBe('/service/rest/internal/cleanup-policies/preview/components/csv');
    expect(Object.fromEntries(url.searchParams)).toEqual({
      repository: 'raw-docs',
      format: 'raw',
      name: 'n',
      criteriaLastDownloaded: '5',
      criteriaAssetRegex: 'docs/.*',
    });
  });

  it('filters by name, format label and description', () => {
    const policy = { name: 'nightly', format: '*', notes: 'Old builds' } as Policy;
    expect(matchesFilter(policy, 'all formats', formatLabel('*', 'All Formats'))).toBe(true);
    expect(matchesFilter(policy, 'old night', 'All Formats')).toBe(true);
    expect(matchesFilter(policy, 'maven', 'All Formats')).toBe(false);
    expect(formatLabel('maven2', 'All')).toBe('Maven');
  });
});
