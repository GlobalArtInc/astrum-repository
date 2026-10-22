import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Markdown, parseMarkdown, safeHref } from './markdown';

describe('the Markdown preview', () => {
  it('parses the usual blocks', () => {
    const source = [
      '# Install',
      '',
      'Run the **installer** with `sudo`.',
      '',
      '```bash',
      'curl -fLO https://example.org/install.sh',
      '```',
      '',
      '- one',
      '- two',
      '  - nested',
      '',
      '1. first',
      '2. second',
      '',
      '> a note',
      '',
      '| Name | Value |',
      '| --- | --- |',
      '| a | 1 |',
      '',
      '---',
    ].join('\n');
    expect(parseMarkdown(source).map((block) => block.kind)).toEqual([
      'heading',
      'paragraph',
      'code',
      'list',
      'list',
      'quote',
      'table',
      'rule',
    ]);
  });

  it('renders text, code, lists and tables', () => {
    render(
      <Markdown
        source={[
          '# Install',
          'Run the **installer** with `sudo` and read [the guide](https://example.org/guide).',
          '',
          '```',
          'echo <b>hi</b>',
          '```',
          '',
          '- one',
          '- two',
          '  - nested',
          '',
          '| Name | Value |',
          '|------|-------|',
          '| a    | 1     |',
        ].join('\n')}
      />,
    );
    expect(screen.getByRole('heading', { name: 'Install' })).toBeInTheDocument();
    expect(screen.getByText('installer').tagName).toBe('STRONG');
    expect(screen.getByText('sudo').tagName).toBe('CODE');
    const guide = screen.getByRole('link', { name: 'the guide' });
    expect(guide).toHaveAttribute('href', 'https://example.org/guide');
    expect(guide).toHaveAttribute('rel', 'noopener noreferrer nofollow');
    expect(screen.getByText('echo <b>hi</b>').tagName).toBe('CODE');
    const lists = screen.getAllByRole('list');
    expect(within(lists[0] as HTMLElement).getByText('nested')).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: '1' })).toBeInTheDocument();
  });

  it('keeps scripts, unsafe links and remote images out', () => {
    const { container } = render(
      <Markdown
        source={
          '<script>alert(1)</script>\n\n[click](javascript:alert(1)) ![pixel](https://tracker.example/p.png)'
        }
      />,
    );
    expect(container.querySelector('script')).toBeNull();
    expect(container.querySelector('img')).toBeNull();
    expect(screen.queryByRole('link', { name: 'click' })).not.toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'pixel' })).toHaveAttribute(
      'href',
      'https://tracker.example/p.png',
    );
    expect(container.textContent).toContain('<script>alert(1)</script>');
    expect(safeHref('mailto:ops@example.org')).toBe('mailto:ops@example.org');
    expect(safeHref('/relative')).toBeNull();
  });
});
