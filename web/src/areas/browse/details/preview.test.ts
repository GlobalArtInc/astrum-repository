import { describe, expect, it } from 'vitest';
import { contentPath, decodeText, IMAGE_LIMIT, linesOf, previewKind, TEXT_LIMIT } from './preview';

describe('file previews', () => {
  it('pick a view from the content type, the name and the size', () => {
    expect(previewKind('text/markdown', 'INSTALL.md', 3700)).toBe('markdown');
    expect(previewKind('application/octet-stream', 'README.md', 10)).toBe('markdown');
    expect(previewKind('text/plain; charset=utf-8', 'notes.txt', 10)).toBe('text');
    expect(previewKind('application/octet-stream', 'install.env.example', 10)).toBe('text');
    expect(previewKind('application/vnd.docker.distribution.manifest.v2+json', 'latest', 900)).toBe('text');
    expect(previewKind('image/png', 'logo.png', 2048)).toBe('image');
    expect(previewKind('image/svg+xml', 'logo.svg', 2048)).toBe('image');
    expect(previewKind('image/png', 'huge.png', IMAGE_LIMIT + 1)).toBe('large');
    expect(previewKind('image/tiff', 'scan.tiff', 10)).toBe('binary');
    expect(previewKind('application/java-archive', 'lib.jar', 10)).toBe('binary');
    expect(previewKind('application/x-gzip', 'app.tgz', 10)).toBe('binary');
    expect(previewKind('text/plain', 'big.log', TEXT_LIMIT + 1)).toBe('large');
    expect(previewKind('application/octet-stream', 'layer', TEXT_LIMIT + 1)).toBe('binary');
  });

  it('decode UTF-8 text and refuse binary content', () => {
    const encode = (value: string) => new TextEncoder().encode(value).buffer as ArrayBuffer;
    expect(decodeText(encode('привет\n'))).toBe('привет\n');
    expect(decodeText(encode('a\u0000b'))).toBeNull();
    expect(decodeText(new Uint8Array([0xff, 0xfe, 0x00]).buffer)).toBeNull();
  });

  it('address the content route and split lines', () => {
    expect(contentPath('raw onprem', '/0.2.2/INSTALL.md')).toBe(
      '/service/rest/internal/ui/browse/raw%20onprem/content?path=%2F0.2.2%2FINSTALL.md',
    );
    expect(linesOf('a\r\nb\n')).toEqual(['a', 'b']);
    expect(linesOf('a\n\n')).toEqual(['a', '']);
    expect(linesOf('')).toEqual(['']);
  });
});
