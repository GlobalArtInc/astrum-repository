import type { BadgeProps } from '@radix-ui/themes';

type Color = NonNullable<BadgeProps['color']>;

export const FORMATS: Record<string, { name: string; color: Color; label?: string }> = {
  alpine: { name: 'Alpine', color: 'cyan' },
  apt: { name: 'APT', color: 'crimson' },
  cargo: { name: 'Cargo', color: 'orange' },
  cocoapods: { name: 'CocoaPods', color: 'red' },
  composer: { name: 'Composer', color: 'brown' },
  conan: { name: 'Conan', color: 'sky' },
  conda: { name: 'Conda', color: 'grass' },
  docker: { name: 'Docker', color: 'indigo' },
  gitlfs: { name: 'Git LFS', color: 'orange', label: 'git-lfs' },
  go: { name: 'Go', color: 'teal' },
  helm: { name: 'Helm', color: 'sky' },
  huggingface: { name: 'Hugging Face', color: 'amber' },
  maven2: { name: 'Maven', color: 'orange', label: 'maven' },
  npm: { name: 'npm', color: 'red' },
  nuget: { name: 'NuGet', color: 'purple' },
  p2: { name: 'P2', color: 'gray' },
  pypi: { name: 'PyPI', color: 'amber' },
  r: { name: 'R', color: 'sky' },
  raw: { name: 'Raw', color: 'gray' },
  rubygems: { name: 'RubyGems', color: 'red' },
  swift: { name: 'Swift', color: 'orange' },
  terraform: { name: 'Terraform', color: 'purple' },
  yum: { name: 'Yum', color: 'green' },
};

export function formatName(id: string): string {
  return FORMATS[id]?.name ?? id;
}

export function formatLabel(id: string): string {
  return FORMATS[id]?.label ?? id;
}

export function formatColor(id: string): Color {
  return FORMATS[id]?.color ?? 'gray';
}
