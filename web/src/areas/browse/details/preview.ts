import { path, withQuery } from '../../../api';
import { UI_PATH } from '../types';

export type PreviewKind = 'image' | 'markdown' | 'text' | 'binary' | 'large';

export const TEXT_LIMIT = 1024 * 1024;
export const IMAGE_LIMIT = 8 * 1024 * 1024;

const IMAGES = [
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
  'image/avif',
  'image/bmp',
  'image/x-icon',
  'image/svg+xml',
];

const BINARY_PREFIXES = ['audio/', 'video/', 'font/', 'image/', 'application/vnd.oci.image.layer'];

const BINARY_TYPES = new Set([
  'application/zip',
  'application/gzip',
  'application/x-gzip',
  'application/x-tar',
  'application/x-compressed-tar',
  'application/x-bzip2',
  'application/x-xz',
  'application/zstd',
  'application/x-7z-compressed',
  'application/vnd.rar',
  'application/java-archive',
  'application/x-java-archive',
  'application/pdf',
  'application/wasm',
  'application/x-executable',
  'application/x-mach-binary',
  'application/x-msdownload',
  'application/x-msi',
  'application/vnd.debian.binary-package',
  'application/x-debian-package',
  'application/x-rpm',
  'application/x-redhat-package-manager',
  'application/vnd.android.package-archive',
  'application/vnd.docker.image.rootfs.diff.tar.gzip',
]);

const MARKDOWN_NAME = /\.(md|markdown|mdown|mkd)$/i;
const TEXT_TYPE =
  /^text\/|[/+](json|xml|yaml|x-yaml|toml|javascript|x-sh|x-shellscript|sql|csv|x-properties)$/;
const TEXT_NAME =
  /(^|\/)(dockerfile|makefile|license|notice|readme|changelog|authors|codeowners)$|\.(md|markdown|txt|text|log|json|ya?ml|xml|toml|ini|cfg|conf|properties|env|example|sh|bash|zsh|ps1|bat|cmd|py|rb|go|rs|java|kts?|gradle|scala|groovy|m?js|cjs|tsx?|jsx|css|scss|html?|sql|csv|tsv|proto|tf|hcl|nix|lock|pom|nuspec|spec)$/i;

export function essenceOf(contentType: string): string {
  return (contentType.split(';')[0] ?? '').trim().toLowerCase();
}

export function previewKind(contentType: string, name: string, size: number): PreviewKind {
  const essence = essenceOf(contentType);
  if (IMAGES.includes(essence)) {
    return size > IMAGE_LIMIT ? 'large' : 'image';
  }
  if (BINARY_TYPES.has(essence) || BINARY_PREFIXES.some((prefix) => essence.startsWith(prefix))) {
    return 'binary';
  }
  if (size > TEXT_LIMIT) {
    return TEXT_TYPE.test(essence) || TEXT_NAME.test(name) ? 'large' : 'binary';
  }
  if (essence === 'text/markdown' || essence === 'text/x-markdown' || MARKDOWN_NAME.test(name)) {
    return 'markdown';
  }
  return 'text';
}

export function decodeText(bytes: ArrayBuffer): string | null {
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    return text.includes('\u0000') ? null : text;
  } catch {
    return null;
  }
}

export function contentPath(repository: string, file: string): string {
  return withQuery(`${UI_PATH}${path`/browse/${repository}/content`}`, { path: file });
}

export function linesOf(text: string): string[] {
  const lines = text.split(/\r\n|\n|\r/);
  if (lines.length > 1 && lines[lines.length - 1] === '') {
    lines.pop();
  }
  return lines;
}
