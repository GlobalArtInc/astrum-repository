import { describe, expect, it } from 'vitest';
import { sanitizeHtml } from './sanitize';

describe('branding snippets', () => {
  it('keep harmless markup and drop scripts, handlers and dangerous links', () => {
    expect(
      sanitizeHtml(
        '<div class="brand" onclick="steal()"><b>Team</b> <a href="javascript:alert(1)">x</a><script>alert(1)</script></div>',
      ),
    ).toBe('<div class="brand"><b>Team</b> <a>x</a></div>');
    expect(sanitizeHtml('<img src="/logo.png" onerror="x()" alt="logo"><iframe src="/x"></iframe>')).toBe(
      '<img src="/logo.png" alt="logo">',
    );
    expect(sanitizeHtml('<a href="https://example.com" target="_blank">site</a>')).toBe(
      '<a href="https://example.com" target="_blank" rel="noopener noreferrer">site</a>',
    );
    expect(sanitizeHtml('<custom-tag>text <u>under</u></custom-tag>')).toBe('text <u>under</u>');
    expect(sanitizeHtml('<span style="background:url(javascript:x)">s</span>')).toBe('<span>s</span>');
  });

  it('see through links hidden behind whitespace, control characters and entities', () => {
    expect(sanitizeHtml('<a href="java&#10;script:alert(1)">a</a>')).toBe('<a>a</a>');
    expect(sanitizeHtml('<a href=" &#1;JavaScript:alert(1)">b</a>')).toBe('<a>b</a>');
    expect(sanitizeHtml('<a href="vbscript:x">c</a><img src="data:text/html;base64,AAAA">')).toBe(
      '<a>c</a><img>',
    );
    expect(sanitizeHtml('<a href="/browse/x:y">d</a><a href="docs/a:b">e</a>')).toBe(
      '<a href="/browse/x:y">d</a><a href="docs/a:b">e</a>',
    );
  });
});
