

import React from 'react';
import {
  SiApachemaven,
  SiNpm,
  SiDocker,
  SiNuget,
  SiPython,
  SiRubygems,
  SiKubernetes,
  SiGo,
  SiTerraform,
  SiCocoapods,
  SiR,
  SiConan,
  SiAnaconda,
  SiRust,
  SiComposer,
  SiHuggingface,
  SiGitlfs,
  SiEclipseide,
  SiAlpinelinux,
  SiDebian,
  SiRedhat,
  SiSwift,
  SiDart,
} from '@icons-pack/react-simple-icons';
import {
  Package,
  Box as LucideBox,
  Container,
  Archive,
  FileCode,
  Folder,
  FileArchive,
  Layers,
  Cloud,
  HardDrive,
  Terminal,
  GitBranch,
  Boxes,
  Workflow,
  Database,
  Code,
  Gem,
  FolderSync,
} from 'lucide-react';
import ansibleGalaxyLogo from '../../../../art/logos/formats/ansible-galaxy.png';

/**
 * Type alias for icon component props.
 * Reduces repetition across FORMAT_SVGS, FORMAT_ICONS, and TYPE_ICONS.
 *
 * `color` is passed to brand SVGs from @icons-pack/react-simple-icons to render
 * each icon in its default brand color (matching the previous CDN behavior).
 * Lucide fallback icons accept `color` as a CSS color string and ignore "default".
 */
export type IconComponent = React.ComponentType<{
  size?: number;
  className?: string;
  color?: string;
  title?: string;
  'aria-hidden'?: boolean | 'true' | 'false';
  'aria-label'?: string;
}>;

/**
 * Bundled brand logo SVG components for repository formats.
 * Sourced from @icons-pack/react-simple-icons (Simple Icons library).
 *
 * Note: 'raw' format intentionally has no brand icon - it uses the Folder fallback.
 * 'ansiblegalaxy' uses a local PNG image instead of an SVG (see FORMAT_IMAGES).
 */
export const FORMAT_SVGS: Record<string, IconComponent> = {
  maven2: SiApachemaven,
  npm: SiNpm,
  docker: SiDocker,
  nuget: SiNuget,
  pypi: SiPython,
  rubygems: SiRubygems,
  helm: SiKubernetes,
  go: SiGo,
  terraform: SiTerraform,
  cocoapods: SiCocoapods,
  r: SiR,
  conan: SiConan,
  conda: SiAnaconda,
  cargo: SiRust,
  composer: SiComposer,
  huggingface: SiHuggingface,
  gitlfs: SiGitlfs,
  p2: SiEclipseide,
  alpine: SiAlpinelinux,
  apt: SiDebian,
  yum: SiRedhat,
  swift: SiSwift,
  pub: SiDart,
};

/**
 * Local image assets for repository formats.
 * Currently only ansiblegalaxy uses a local PNG; all other formats use SVGs or lucide fallbacks.
 */
export const FORMAT_IMAGES: Record<string, string> = {
  ansiblegalaxy: ansibleGalaxyLogo,
};

/**
 * Fallback Lucide icons for repository formats.
 * Used when useBrandLogo is false, or when no SVG/image is available for the format.
 *
 * Note: 'raw' format has no brand identity, so it always uses the Folder icon.
 */
export const FORMAT_ICONS: Record<string, IconComponent> = {
  maven2: Package,
  npm: LucideBox,
  docker: Container,
  nuget: Archive,
  pypi: FileCode,
  rubygems: Gem,
  raw: Folder,
  helm: Layers,
  go: Terminal,
  yum: FileArchive,
  apt: FileArchive,
  conan: Boxes,
  conda: Cloud,
  cocoapods: HardDrive,
  gitlfs: GitBranch,
  p2: Workflow,
  r: FileCode,
  composer: Package,
  cargo: Package,
  huggingface: Cloud,
  terraform: Layers,
  alpine: Package,
  swift: Code,
  pub: Package,
  ansiblegalaxy: Boxes,
};

/**
 * Icons for repository types (hosted, proxy, group).
 */
export const TYPE_ICONS: Record<string, IconComponent> = {
  proxy: Cloud,
  hosted: Database,
  group: FolderSync,
};

/**
 * Default fallback icon for unknown formats.
 */
export const DEFAULT_FORMAT_ICON: IconComponent = Package;
