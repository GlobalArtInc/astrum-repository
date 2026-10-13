import { describe, expect, it } from 'vitest';
import {
  countNodes,
  describePattern,
  escapeRegex,
  fromRule,
  matcher,
  matchesFilter,
  normalizePath,
  type PreviewNode,
  sameValues,
  toPattern,
  toRequest,
  validate,
} from './model';

const TEXTS = {
  nameRequired: 'name required',
  namePattern: 'name pattern',
  nameLength: 'name length',
  matchersRequired: 'matchers required',
};

function node(repository: string, allowed: boolean, children: PreviewNode[] | null = null): PreviewNode {
  return {
    repository,
    type: children ? 'group' : 'proxy',
    format: 'raw',
    rule: null,
    allowed,
    expanded: children !== null,
    expandable: children !== null,
    children,
  };
}

describe('routing rule matchers', () => {
  it('turn the typed kinds into anchored regular expressions', () => {
    expect(toPattern({ kind: 'prefix', text: '/org/apache/' })).toBe('^/org/apache/.*');
    expect(toPattern({ kind: 'suffix', text: '.jar' })).toBe('.*\\.jar');
    expect(toPattern({ kind: 'contains', text: 'a+b' })).toBe('.*a\\+b.*');
    expect(toPattern({ kind: 'regex', text: '(a|b)' })).toBe('(a|b)');
    expect(escapeRegex('a.b*c?(d)[e]{f}|g^h$i\\')).toBe('a\\.b\\*c\\?\\(d\\)\\[e\\]\\{f\\}\\|g\\^h\\$i\\\\');
  });

  it('read simple expressions back as words for the summary', () => {
    expect(describePattern('^/blocked/.*')).toEqual({ kind: 'prefix', text: '/blocked/' });
    expect(describePattern('.*/secret/.*')).toEqual({ kind: 'contains', text: '/secret/' });
    expect(describePattern('.*\\.jar')).toEqual({ kind: 'suffix', text: '.jar' });
    expect(describePattern('.*-sources\\.jar$')).toEqual({ kind: 'suffix', text: '-sources.jar' });
    expect(describePattern('com/example/.*')).toEqual({ kind: 'regex', text: 'com/example/.*' });
    expect(describePattern('secret')).toEqual({ kind: 'regex', text: 'secret' });
    for (const kind of ['prefix', 'suffix', 'contains'] as const) {
      expect(describePattern(toPattern({ kind, text: 'x.y' }))).toEqual({ kind, text: 'x.y' });
    }
  });

  it('send only filled matchers and compare forms by what they send', () => {
    const values = {
      name: ' rule ',
      description: ' d ',
      mode: 'ALLOW' as const,
      matchers: [matcher('prefix', '/org/'), matcher('regex', '  '), matcher('regex', 'x')],
    };
    expect(toRequest(values)).toEqual({
      name: 'rule',
      description: 'd',
      mode: 'ALLOW',
      matchers: ['^/org/.*', 'x'],
    });
    const loaded = fromRule({ name: 'rule', description: 'd', mode: 'ALLOW', matchers: ['^/org/.*', 'x'] });
    expect(loaded.matchers.map((entry) => entry.kind)).toEqual(['regex', 'regex']);
    expect(sameValues(values, loaded)).toBe(true);
    expect(fromRule({ name: 'r', description: '', mode: 'BLOCK', matchers: [] }).matchers).toHaveLength(1);
  });

  it('check the name and that a matcher exists', () => {
    const base = { name: 'ok-1', description: '', mode: 'BLOCK' as const, matchers: [matcher('regex', 'x')] };
    expect(validate(base, TEXTS)).toEqual({});
    expect(validate({ ...base, name: ' ' }, TEXTS).name).toBe('name required');
    expect(validate({ ...base, name: '_bad' }, TEXTS).name).toBe('name pattern');
    expect(validate({ ...base, name: 'a'.repeat(201) }, TEXTS).name).toBe('name length');
    expect(validate({ ...base, matchers: [matcher()] }, TEXTS).matchers).toBe('matchers required');
  });

  it('filter by name or description and give paths a leading slash', () => {
    const rule = {
      name: 'Block-Sources',
      description: 'No source jars',
      mode: 'BLOCK' as const,
      matchers: [],
    };
    expect(matchesFilter(rule, 'sources')).toBe(true);
    expect(matchesFilter(rule, 'JARS')).toBe(true);
    expect(matchesFilter(rule, 'maven')).toBe(false);
    expect(normalizePath(' a/b ')).toBe('/a/b');
    expect(normalizePath('/a')).toBe('/a');
    expect(normalizePath('  ')).toBe('');
  });

  it('count every repository of the preview tree', () => {
    const tree = [node('g', true, [node('h', true), node('p', false)]), node('p', false), node('q', true)];
    expect(countNodes(tree)).toEqual({ total: 5, allowed: 3, blocked: 2 });
  });
});
