import type { Translate } from '../../i18n';

type Token =
  | { kind: 'word'; text: string }
  | { kind: 'operator'; text: string }
  | { kind: 'string'; text: string }
  | { kind: 'join'; text: 'and' | 'or' }
  | { kind: 'paren'; text: string };

export function tokenize(expression: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;
  while (index < expression.length) {
    const rest = expression.slice(index);
    const space = rest.match(/^\s+/);
    if (space) {
      index += space[0].length;
      continue;
    }
    const operator = rest.match(/^(==|!=|=~|=\^)/);
    if (operator) {
      tokens.push({ kind: 'operator', text: operator[0] });
      index += operator[0].length;
      continue;
    }
    if (rest.startsWith('&&') || rest.startsWith('||')) {
      tokens.push({ kind: 'join', text: rest.startsWith('&&') ? 'and' : 'or' });
      index += 2;
      continue;
    }
    const first = rest[0] ?? '';
    if (first === '(' || first === ')') {
      tokens.push({ kind: 'paren', text: first });
      index += 1;
      continue;
    }
    if (first === '"' || first === "'") {
      let end = 1;
      let text = '';
      while (end < rest.length && rest[end] !== first) {
        if (rest[end] === '\\' && end + 1 < rest.length) {
          end += 1;
        }
        text += rest[end];
        end += 1;
      }
      tokens.push({ kind: 'string', text });
      index += end + 1;
      continue;
    }
    const word = rest.match(/^[^\s()"'=!&|]+/);
    if (word) {
      const lower = word[0].toLowerCase();
      if (lower === 'and' || lower === 'or') {
        tokens.push({ kind: 'join', text: lower });
      } else {
        tokens.push({ kind: 'word', text: word[0] });
      }
      index += word[0].length;
      continue;
    }
    tokens.push({ kind: 'word', text: first });
    index += 1;
  }
  return tokens;
}

const OPERATORS: Record<string, string> = {
  '==': 'selectors.explain.equals',
  '!=': 'selectors.explain.notEquals',
  '=~': 'selectors.explain.matches',
  '=^': 'selectors.explain.startsWith',
};

export function explainExpression(t: Translate, expression: string): string {
  const parts = tokenize(expression).map((token) => {
    switch (token.kind) {
      case 'operator':
        return t(OPERATORS[token.text] ?? 'selectors.explain.equals');
      case 'string':
        return t('selectors.explain.value', { value: token.text });
      case 'join':
        return token.text.toUpperCase();
      default:
        return token.text;
    }
  });
  return t('selectors.explain.prefix', { condition: parts.join(' ') });
}
