import { createElement, type ReactNode } from 'react';
import styles from './Markdown.module.css';

export type Block =
  | { kind: 'heading'; level: number; text: string }
  | { kind: 'paragraph'; text: string }
  | { kind: 'code'; text: string; language: string }
  | { kind: 'quote'; blocks: Block[] }
  | { kind: 'list'; ordered: boolean; start: number; items: Block[][] }
  | { kind: 'table'; head: string[]; rows: string[][] }
  | { kind: 'rule' };

const FENCE = /^ {0,3}(`{3,}|~{3,})\s*([^\s`]*)/;
const HEADING = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const RULE = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;
const QUOTE = /^ {0,3}>[ ]?(.*)$/;
const ITEM = /^( *)([-*+]|\d{1,9}[.)])(?:[ \t]+(.*))?$/;
const TABLE_RULE = /^[ \t]*\|?[ \t]*:?-+:?[ \t]*(\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$/;

function leading(line: string): number {
  return line.length - line.trimStart().length;
}

function startsBlock(line: string): boolean {
  return FENCE.test(line) || HEADING.test(line) || RULE.test(line) || QUOTE.test(line) || ITEM.test(line);
}

function cells(line: string): string[] {
  let row = line.trim();
  if (row.startsWith('|')) {
    row = row.slice(1);
  }
  if (row.endsWith('|') && !row.endsWith('\\|')) {
    row = row.slice(0, -1);
  }
  return row.split(/(?<!\\)\|/).map((cell) => cell.trim().replace(/\\\|/g, '|'));
}

function closes(line: string, marker: string): boolean {
  const trimmed = line.trim();
  return trimmed.length >= marker.length && trimmed === marker.charAt(0).repeat(trimmed.length);
}

function listAt(lines: string[], start: number): [Block, number] {
  const at = (index: number) => lines[index] ?? '';
  const first = ITEM.exec(at(start));
  const indent = first?.[1]?.length ?? 0;
  const marker = first?.[2] ?? '-';
  const ordered = /\d/.test(marker);
  const items: string[][] = [];
  let offset = 0;
  let i = start;
  while (i < lines.length) {
    const line = at(i);
    const item = ITEM.exec(line);
    const itemIndent = item?.[1]?.length ?? 0;
    const itemMarker = item?.[2] ?? '';
    if (item && itemIndent <= indent) {
      if (/\d/.test(itemMarker) !== ordered) {
        break;
      }
      items.push([item[3] ?? '']);
      offset = itemIndent + itemMarker.length + 1;
      i += 1;
      continue;
    }
    const current = items[items.length - 1] ?? [];
    if (line.trim() === '') {
      const next = lines[i + 1];
      if (next !== undefined && next.trim() !== '' && leading(next) > indent) {
        current.push('');
        i += 1;
        continue;
      }
      break;
    }
    if (leading(line) > indent) {
      current.push(line.slice(Math.min(leading(line), offset)));
      i += 1;
      continue;
    }
    if (current[current.length - 1] !== '' && !startsBlock(line)) {
      current.push(line.trim());
      i += 1;
      continue;
    }
    break;
  }
  return [
    {
      kind: 'list',
      ordered,
      start: ordered ? Number.parseInt(marker, 10) : 1,
      items: items.map((item) => blocks(item)),
    },
    i,
  ];
}

function blocks(lines: string[]): Block[] {
  const at = (index: number) => lines[index] ?? '';
  const out: Block[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = at(i);
    if (line.trim() === '') {
      i += 1;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const marker = fence[1] ?? '```';
      const body: string[] = [];
      const indent = leading(line);
      i += 1;
      while (i < lines.length && !closes(at(i), marker)) {
        body.push(at(i).slice(Math.min(leading(at(i)), indent)));
        i += 1;
      }
      i += 1;
      out.push({ kind: 'code', text: body.join('\n'), language: fence[2] ?? '' });
      continue;
    }
    const heading = HEADING.exec(line);
    if (heading) {
      out.push({ kind: 'heading', level: heading[1]?.length ?? 1, text: heading[2] ?? '' });
      i += 1;
      continue;
    }
    if (RULE.test(line)) {
      out.push({ kind: 'rule' });
      i += 1;
      continue;
    }
    if (QUOTE.test(line)) {
      const body: string[] = [];
      while (i < lines.length && at(i).trim() !== '') {
        body.push(QUOTE.exec(at(i))?.[1] ?? at(i));
        i += 1;
      }
      out.push({ kind: 'quote', blocks: blocks(body) });
      continue;
    }
    if (ITEM.test(line)) {
      const [list, next] = listAt(lines, i);
      out.push(list);
      i = next;
      continue;
    }
    const rule = lines[i + 1];
    if (line.includes('|') && rule?.includes('-') && TABLE_RULE.test(rule)) {
      const head = cells(line);
      const rows: string[][] = [];
      i += 2;
      while (i < lines.length && at(i).trim() !== '' && at(i).includes('|')) {
        rows.push(cells(at(i)));
        i += 1;
      }
      out.push({ kind: 'table', head, rows });
      continue;
    }
    const text = [line.trim()];
    i += 1;
    while (i < lines.length && at(i).trim() !== '' && !startsBlock(at(i))) {
      text.push(at(i).trim());
      i += 1;
    }
    out.push({ kind: 'paragraph', text: text.join('\n') });
  }
  return out;
}

export function parseMarkdown(source: string): Block[] {
  const lines = source
    .replace(/\r\n?/g, '\n')
    .split('\n')
    .map((line) => line.replace(/^\t+/, (tabs) => '    '.repeat(tabs.length)));
  return blocks(lines);
}

export function safeHref(url: string): string | null {
  return /^(https?:\/\/|mailto:)/i.test(url) ? url : null;
}

const INLINE = new RegExp(
  [
    '(?<code>`+)(?<codeText>[\\s\\S]*?[^`])\\k<code>(?!`)',
    '!\\[(?<alt>[^\\]]*)\\]\\((?<src>[^)\\s]+)(?:\\s+"[^"]*")?\\)',
    '\\[(?<label>[^\\]]+)\\]\\((?<href>[^)\\s]+)(?:\\s+"[^"]*")?\\)',
    '<(?<auto>(?:https?:\\/\\/|mailto:)[^\\s>]+)>',
    '(?<bare>https?:\\/\\/[^\\s<]*[^\\s<.,;:!?)\\]\'"])',
    '\\*\\*(?<strong>[^\\s*](?:[\\s\\S]*?[^\\s])?)\\*\\*',
    '(?<![\\w])__(?<strongLine>[^\\s_](?:[\\s\\S]*?[^\\s])?)__(?![\\w])',
    '~~(?<strike>[^\\s~](?:[\\s\\S]*?[^\\s])?)~~',
    '\\*(?<em>[^\\s*](?:[\\s\\S]*?[^\\s*])?)\\*',
    '(?<![\\w])_(?<emLine>[^\\s_](?:[\\s\\S]*?[^\\s_])?)_(?![\\w])',
    '\\\\(?<escaped>[\\\\`*_{}\\[\\]()#+\\-.!|>~])',
    '(?<hard>(?: {2,}|\\\\)\\n)',
    '(?<soft>\\n)',
  ].join('|'),
  'g',
);

function link(href: string, children: ReactNode, key: string): ReactNode {
  return (
    <a key={key} href={href} target="_blank" rel="noopener noreferrer nofollow">
      {children}
    </a>
  );
}

export function inline(text: string, prefix = 'i'): ReactNode[] {
  const out: ReactNode[] = [];
  let last = 0;
  let index = 0;
  for (const match of text.matchAll(INLINE)) {
    const start = match.index ?? 0;
    if (start > last) {
      out.push(text.slice(last, start));
    }
    last = start + match[0].length;
    const key = `${prefix}-${index}`;
    index += 1;
    const group = match.groups ?? {};
    if (group.code !== undefined) {
      const code = group.codeText ?? '';
      out.push(<code key={key}>{code.trim() === '' ? code : code.trim()}</code>);
    } else if (group.src !== undefined) {
      const href = safeHref(group.src);
      const shown = group.alt ? group.alt : group.src;
      out.push(href ? link(href, shown, key) : shown);
    } else if (group.href !== undefined) {
      const href = safeHref(group.href);
      const children = inline(group.label ?? '', key);
      out.push(href ? link(href, children, key) : <span key={key}>{children}</span>);
    } else if (group.auto !== undefined) {
      out.push(link(group.auto, group.auto.replace(/^mailto:/i, ''), key));
    } else if (group.bare !== undefined) {
      out.push(link(group.bare, group.bare, key));
    } else if (group.strong !== undefined || group.strongLine !== undefined) {
      out.push(<strong key={key}>{inline(group.strong ?? group.strongLine ?? '', key)}</strong>);
    } else if (group.strike !== undefined) {
      out.push(<del key={key}>{inline(group.strike, key)}</del>);
    } else if (group.em !== undefined || group.emLine !== undefined) {
      out.push(<em key={key}>{inline(group.em ?? group.emLine ?? '', key)}</em>);
    } else if (group.escaped !== undefined) {
      out.push(group.escaped);
    } else if (group.hard !== undefined) {
      out.push(<br key={key} />);
    } else {
      out.push(' ');
    }
  }
  if (last < text.length) {
    out.push(text.slice(last));
  }
  return out;
}

function render(items: Block[], prefix: string): ReactNode[] {
  return items.map((block, index) => {
    const key = `${prefix}-${index}`;
    switch (block.kind) {
      case 'heading':
        return createElement(
          `h${Math.min(block.level + 2, 6)}`,
          { key, 'data-level': block.level },
          inline(block.text, key),
        );
      case 'paragraph':
        return <p key={key}>{inline(block.text, key)}</p>;
      case 'code':
        return (
          <pre key={key} data-language={block.language || undefined}>
            <code>{block.text}</code>
          </pre>
        );
      case 'quote':
        return <blockquote key={key}>{render(block.blocks, key)}</blockquote>;
      case 'list': {
        const items = block.items.map((item, position) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: list items have no identity of their own
          <li key={`${key}-${position}`}>{render(item, `${key}-${position}`)}</li>
        ));
        return block.ordered ? (
          <ol key={key} start={block.start === 1 ? undefined : block.start}>
            {items}
          </ol>
        ) : (
          <ul key={key}>{items}</ul>
        );
      }
      case 'table':
        return (
          <div key={key} className={styles.tableWrap}>
            <table>
              <thead>
                <tr>
                  {block.head.map((cell, column) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: columns are positional
                    <th key={column}>{inline(cell, `${key}-h${column}`)}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {block.rows.map((row, line) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: rows are positional
                  <tr key={line}>
                    {block.head.map((_, column) => (
                      // biome-ignore lint/suspicious/noArrayIndexKey: columns are positional
                      <td key={column}>{inline(row[column] ?? '', `${key}-${line}-${column}`)}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
      default:
        return <hr key={key} />;
    }
  });
}

export function Markdown({ source }: { source: string }) {
  return <div className={styles.markdown}>{render(parseMarkdown(source), 'b')}</div>;
}
