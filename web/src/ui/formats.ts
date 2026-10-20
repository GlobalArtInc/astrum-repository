import type { BadgeProps } from '@radix-ui/themes';

type Color = NonNullable<BadgeProps['color']>;

export const FORMATS: Record<string, { name: string; color: Color; label?: string }> = {
  alpine: { name: 'Alpine', color: 'cyan' },
  ansible: { name: 'Ansible Galaxy', color: 'ruby' },
  apt: { name: 'APT', color: 'green' },
  cargo: { name: 'Cargo (Rust)', color: 'orange' },
  chef: { name: 'Chef Supermarket', color: 'orange' },
  cocoapods: { name: 'CocoaPods', color: 'red' },
  composer: { name: 'Composer', color: 'purple' },
  conan: { name: 'Conan', color: 'cyan' },
  conda: { name: 'Conda', color: 'grass' },
  docker: { name: 'Docker', color: 'indigo' },
  gitlfs: { name: 'Git LFS', color: 'orange', label: 'git-lfs' },
  go: { name: 'Go', color: 'cyan' },
  helm: { name: 'Helm', color: 'sky' },
  huggingface: { name: 'Hugging Face', color: 'amber' },
  maven2: { name: 'Maven', color: 'orange', label: 'maven' },
  nix: { name: 'Nix', color: 'blue' },
  npm: { name: 'npm', color: 'red' },
  nuget: { name: 'NuGet', color: 'purple' },
  p2: { name: 'P2', color: 'gray' },
  pub: { name: 'pub (Dart)', color: 'sky' },
  puppet: { name: 'Puppet Forge', color: 'amber' },
  pypi: { name: 'PyPI', color: 'amber' },
  r: { name: 'R', color: 'sky' },
  raw: { name: 'Raw', color: 'gray' },
  rubygems: { name: 'RubyGems', color: 'red' },
  swift: { name: 'Swift', color: 'orange' },
  terraform: { name: 'Terraform', color: 'purple' },
  vagrant: { name: 'Vagrant', color: 'blue' },
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
