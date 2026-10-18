import { INTERNAL, V1 } from '../../api';

export const ADMIN = 'astrum:*';
export const RULES = `${V1}/routing-rules`;
export const UI_RULES = `${INTERNAL}/ui/routing-rules`;
export const TEST = `${UI_RULES}/test`;
export const PREVIEW = `${UI_RULES}/preview`;
export const LIST_PAGE = '/admin/repository/routing-rules';
export const PREVIEW_PAGE = `${LIST_PAGE}/preview`;
export const CREATE_PAGE = `${LIST_PAGE}/create`;
export const DOCS = 'https://links.globalart.dev/products/astrum/docs/routing-rules';
export const NAME_PATTERN = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;
export const MAX_NAME = 200;

export type Mode = 'ALLOW' | 'BLOCK';
export type MatcherKind = 'regex' | 'prefix' | 'suffix' | 'contains';
export type Filter = 'all' | 'groups' | 'proxies';

export interface RoutingRule {
  name: string;
  description: string;
  mode: Mode;
  matchers: string[];
}

export interface RoutingRuleDetails extends RoutingRule {
  assignedRepositoryCount: number;
  assignedRepositoryNames: string[] | null;
}

export interface TestResult {
  allowed: boolean;
  matchers: Array<{ matches: boolean; error: string | null }>;
}

export interface PreviewNode {
  repository: string | null;
  type: string | null;
  format: string | null;
  rule: string | null;
  allowed: boolean | null;
  expanded: boolean;
  expandable: boolean;
  children: PreviewNode[] | null;
}

export interface Matcher {
  id: number;
  kind: MatcherKind;
  text: string;
}

export interface RuleValues {
  name: string;
  description: string;
  mode: Mode;
  matchers: Matcher[];
}

let nextId = 0;

export function matcher(kind: MatcherKind = 'regex', text = ''): Matcher {
  nextId += 1;
  return { id: nextId, kind, text };
}

export const EMPTY_RULE: RuleValues = { name: '', description: '', mode: 'BLOCK', matchers: [] };

export function emptyRule(): RuleValues {
  return { ...EMPTY_RULE, matchers: [matcher()] };
}

export function ruleHref(name: string): string {
  return `${LIST_PAGE}/${encodeURIComponent(name)}`;
}

const META = /[\\^$.|?*+()[\]{}]/g;

export function escapeRegex(text: string): string {
  return text.replace(META, '\\$&');
}

export function toPattern(entry: Pick<Matcher, 'kind' | 'text'>): string {
  const literal = escapeRegex(entry.text);
  switch (entry.kind) {
    case 'prefix':
      return `^${literal}.*`;
    case 'suffix':
      return `.*${literal}`;
    case 'contains':
      return `.*${literal}.*`;
    default:
      return entry.text;
  }
}

const LITERAL = String.raw`((?:[^\\^$.|?*+()[\]{}]|\\[\\^$.|?*+()[\]{}/-])+)`;
const STARTS = new RegExp(`^\\^${LITERAL}\\.\\*$`);
const CONTAINS = new RegExp(`^\\.\\*${LITERAL}\\.\\*$`);
const ENDS = new RegExp(`^\\.\\*${LITERAL}\\$?$`);

function unescapeLiteral(text: string): string {
  return text.replace(/\\(.)/g, '$1');
}

export function describePattern(pattern: string): { kind: MatcherKind; text: string } {
  const starts = STARTS.exec(pattern);
  if (starts?.[1]) {
    return { kind: 'prefix', text: unescapeLiteral(starts[1]) };
  }
  const contains = CONTAINS.exec(pattern);
  if (contains?.[1]) {
    return { kind: 'contains', text: unescapeLiteral(contains[1]) };
  }
  const ends = ENDS.exec(pattern);
  if (ends?.[1]) {
    return { kind: 'suffix', text: unescapeLiteral(ends[1]) };
  }
  return { kind: 'regex', text: pattern };
}

export function fromRule(rule: RoutingRule): RuleValues {
  return {
    name: rule.name,
    description: rule.description ?? '',
    mode: rule.mode,
    matchers:
      rule.matchers.length > 0 ? rule.matchers.map((pattern) => matcher('regex', pattern)) : [matcher()],
  };
}

export function patterns(values: RuleValues): string[] {
  return values.matchers.filter((entry) => entry.text.trim() !== '').map(toPattern);
}

export function toRequest(values: RuleValues): RoutingRule {
  return {
    name: values.name.trim(),
    description: values.description.trim(),
    mode: values.mode,
    matchers: patterns(values),
  };
}

export function sameValues(left: RuleValues, right: RuleValues): boolean {
  return JSON.stringify(toRequest(left)) === JSON.stringify(toRequest(right));
}

export interface Texts {
  nameRequired: string;
  namePattern: string;
  nameLength: string;
  matchersRequired: string;
}

export type Problems = Partial<Record<'name' | 'mode' | 'description' | 'matchers', string>> & {
  matcher?: Record<number, string>;
};

export function validate(values: RuleValues, texts: Texts): Problems {
  const problems: Problems = {};
  const name = values.name.trim();
  if (name === '') {
    problems.name = texts.nameRequired;
  } else if (!NAME_PATTERN.test(name)) {
    problems.name = texts.namePattern;
  } else if (name.length > MAX_NAME) {
    problems.name = texts.nameLength;
  }
  if (patterns(values).length === 0) {
    problems.matchers = texts.matchersRequired;
  }
  return problems;
}

export function matchesFilter(rule: RoutingRule, filter: string): boolean {
  const text = filter.trim().toLowerCase();
  if (text === '') {
    return true;
  }
  return rule.name.toLowerCase().includes(text) || (rule.description ?? '').toLowerCase().includes(text);
}

export function normalizePath(path: string): string {
  const trimmed = path.trim();
  return trimmed === '' || trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
}

export interface Counts {
  total: number;
  allowed: number;
  blocked: number;
}

export function countNodes(nodes: readonly PreviewNode[]): Counts {
  const counts: Counts = { total: 0, allowed: 0, blocked: 0 };
  const visit = (node: PreviewNode) => {
    if (node.repository !== null) {
      counts.total += 1;
      if (node.allowed === false) {
        counts.blocked += 1;
      } else {
        counts.allowed += 1;
      }
    }
    for (const child of node.children ?? []) {
      visit(child);
    }
  };
  for (const node of nodes) {
    visit(node);
  }
  return counts;
}
