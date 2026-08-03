

/**
 * Formats supported by Repository Firewall.
 *
 * Source of truth: FirewallSupportedFormat.java
 *
 * NOTE: p2 and raw are in the backend enum but not in official docs
 * (https://help.globalart.dev/en/repository-firewall.html).
 * They are treated as supported until product team clarification.
 *
 * UNSUPPORTED (show N/A): terraform, apt, helm, gitlfs, swift
 */
export const FIREWALL_SUPPORTED_FORMATS = new Set([
  'r',
  'go',
  'p2',
  'rubygems',
  'npm',
  'yum',
  'pypi',
  'cargo',
  'conan',
  'conda',
  'nuget',
  'maven2',
  'composer',
  'cocoapods',
  'huggingface',
  'docker',
  'raw',
]);

/**
 * Returns true when Repository Firewall supports the given format.
 * Case-insensitive. Returns false for null/undefined/empty.
 */
export function isFirewallSupportedFormat(format: string | null | undefined): boolean {
  return FIREWALL_SUPPORTED_FORMATS.has(format?.toLowerCase?.() ?? '');
}
