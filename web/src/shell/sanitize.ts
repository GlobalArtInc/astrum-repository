const TAGS = new Set([
  'a',
  'abbr',
  'b',
  'br',
  'center',
  'code',
  'div',
  'em',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'hr',
  'i',
  'img',
  'li',
  'ol',
  'p',
  'pre',
  'small',
  'span',
  'strong',
  'sub',
  'sup',
  'table',
  'tbody',
  'td',
  'th',
  'thead',
  'tr',
  'u',
  'ul',
]);
const ATTRIBUTES = new Set([
  'alt',
  'align',
  'class',
  'height',
  'href',
  'rel',
  'src',
  'style',
  'target',
  'title',
  'width',
]);
const DROPPED = new Set([
  'script',
  'style',
  'iframe',
  'object',
  'embed',
  'template',
  'noscript',
  'svg',
  'math',
  'form',
]);

const SCHEMES = new Set(['http', 'https', 'mailto']);

function visible(value: string): string {
  let text = '';
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    if (code > 0x20 && (code < 0x7f || code > 0x9f)) {
      text += char;
    }
  }
  return text;
}

function safeUrl(value: string, image: boolean): boolean {
  const url = visible(value).toLowerCase();
  if (image && /^data:image\/(png|gif|jpe?g|webp);base64,/.test(url)) {
    return true;
  }
  const scheme = /^([a-z][a-z0-9+.-]*):/.exec(url)?.[1];
  return scheme === undefined || SCHEMES.has(scheme);
}

function safeStyle(value: string): boolean {
  return !/expression\s*\(|javascript:|url\s*\(|@import|behavior\s*:/i.test(value);
}

function clean(source: Node, target: Node, document: Document): void {
  for (const child of Array.from(source.childNodes)) {
    if (child.nodeType === Node.TEXT_NODE) {
      target.appendChild(document.createTextNode(child.textContent ?? ''));
      continue;
    }
    if (child.nodeType !== Node.ELEMENT_NODE) {
      continue;
    }
    const element = child as Element;
    const tag = element.tagName.toLowerCase();
    if (DROPPED.has(tag)) {
      continue;
    }
    if (!TAGS.has(tag)) {
      clean(element, target, document);
      continue;
    }
    const copy = document.createElement(tag);
    for (const attribute of Array.from(element.attributes)) {
      const name = attribute.name.toLowerCase();
      if (!ATTRIBUTES.has(name)) {
        continue;
      }
      if ((name === 'href' || name === 'src') && !safeUrl(attribute.value, name === 'src' && tag === 'img')) {
        continue;
      }
      if (name === 'style' && !safeStyle(attribute.value)) {
        continue;
      }
      copy.setAttribute(name, attribute.value);
    }
    if (tag === 'a' && copy.getAttribute('target')) {
      copy.setAttribute('rel', 'noopener noreferrer');
    }
    clean(element, copy, document);
    target.appendChild(copy);
  }
}

export function sanitizeHtml(html: string): string {
  const parsed = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
  const output = document.implementation.createHTMLDocument('');
  const root = output.createElement('div');
  clean(parsed.body, root, output);
  return root.innerHTML;
}
