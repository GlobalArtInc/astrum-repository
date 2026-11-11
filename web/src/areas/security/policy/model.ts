import { path } from '../../../api';

export const POLICIES_PAGE = '/admin/security/policies';

export type Mode = 'off' | 'report' | 'block';
export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'unknown';
export type Malicious = 'block' | 'threshold';
export type Unscanned = 'allow' | 'block';
export type LicenseList = 'allow' | 'deny';
export type UnknownLicense = 'allow' | 'report' | 'block';
export type BlockName = 'vulnerabilities' | 'licenses' | 'signatures';

const LICENSE_LISTS: readonly LicenseList[] = ['allow', 'deny'];
const SEVERITIES: readonly Severity[] = ['critical', 'high', 'medium', 'low', 'unknown'];

export interface VulnerabilityRule {
  mode: Mode;
  severity: Severity | null;
  malicious: Malicious;
  unscanned: Unscanned;
  notify: Severity | null;
}

export interface LicenseRule {
  mode: Mode;
  list: LicenseList | null;
  ids: string[];
  unknown: UnknownLicense;
}

export interface SignatureRule {
  mode: Mode;
  keys: string[];
}

export interface Policy {
  name: string;
  notes: string;
  vulnerabilities: VulnerabilityRule;
  licenses: LicenseRule;
  signatures: SignatureRule;
  revision: number;
}

export interface PolicyForm {
  name: string;
  notes: string;
  vulnerabilityMode: Mode;
  severity: string;
  malicious: Malicious;
  unscanned: Unscanned;
  notify: string;
  licenseMode: Mode;
  licenseList: string;
  licenseIds: string;
  unknownLicense: UnknownLicense;
  signatureMode: Mode;
  signatureKeys: string[];
}

export interface PolicyRequest {
  name?: string;
  notes: string;
  vulnerabilities: VulnerabilityRule;
  licenses: LicenseRule;
  signatures: SignatureRule;
}

export const EMPTY_FORM: PolicyForm = {
  name: '',
  notes: '',
  vulnerabilityMode: 'off',
  severity: '',
  malicious: 'block',
  unscanned: 'allow',
  notify: '',
  licenseMode: 'off',
  licenseList: '',
  licenseIds: '',
  unknownLicense: 'report',
  signatureMode: 'off',
  signatureKeys: [],
};

export const DAY = 24 * 60 * 60 * 1000;

export const NONE = '_none';

export interface TrustedKey {
  name: string;
  algorithm: string;
  publicKey: string;
  fingerprint: string;
  notes: string;
  createdAt: string;
}

export interface Choice<V extends string> {
  value: V;
  label: string;
}

export const MODE_OPTIONS: Array<Choice<Mode>> = [
  { value: 'off', label: 'security.policy.modes.off' },
  { value: 'report', label: 'security.policy.modes.report' },
  { value: 'block', label: 'security.policy.modes.block' },
];

export const SEVERITY_OPTIONS: Array<Choice<Severity>> = [
  { value: 'critical', label: 'security.policy.severity.critical' },
  { value: 'high', label: 'security.policy.severity.high' },
  { value: 'medium', label: 'security.policy.severity.medium' },
  { value: 'low', label: 'security.policy.severity.low' },
  { value: 'unknown', label: 'security.policy.severity.unknown' },
];

export const MALICIOUS_OPTIONS: Array<Choice<Malicious>> = [
  { value: 'block', label: 'security.policy.malicious.block' },
  { value: 'threshold', label: 'security.policy.malicious.threshold' },
];

export const UNSCANNED_OPTIONS: Array<Choice<Unscanned>> = [
  { value: 'allow', label: 'security.policy.unscanned.allow' },
  { value: 'block', label: 'security.policy.unscanned.block' },
];

export const LICENSE_LIST_OPTIONS: Array<Choice<LicenseList>> = [
  { value: 'allow', label: 'security.policy.licenseList.allow' },
  { value: 'deny', label: 'security.policy.licenseList.deny' },
];

export const UNKNOWN_LICENSE_OPTIONS: Array<Choice<UnknownLicense>> = [
  { value: 'allow', label: 'security.policy.unknownLicense.allow' },
  { value: 'report', label: 'security.policy.unknownLicense.report' },
  { value: 'block', label: 'security.policy.unknownLicense.block' },
];

export function policyHref(name: string): string {
  return path`${POLICIES_PAGE}/${name}`;
}

export function severityOf(text: string): Severity | null {
  return SEVERITIES.find((severity) => severity === text) ?? null;
}

function listOf(text: string): LicenseList | null {
  return LICENSE_LISTS.find((list) => list === text) ?? null;
}

export function idsOf(text: string): string[] {
  return text.split(/[\s,]+/).filter((id) => id !== '');
}

export function formOf(policy: Policy): PolicyForm {
  return {
    name: policy.name,
    notes: policy.notes,
    vulnerabilityMode: policy.vulnerabilities.mode,
    severity: policy.vulnerabilities.severity ?? '',
    malicious: policy.vulnerabilities.malicious,
    unscanned: policy.vulnerabilities.unscanned,
    notify: policy.vulnerabilities.notify ?? '',
    licenseMode: policy.licenses.mode,
    licenseList: policy.licenses.list ?? '',
    licenseIds: policy.licenses.ids.join('\n'),
    unknownLicense: policy.licenses.unknown,
    signatureMode: policy.signatures.mode,
    signatureKeys: [...policy.signatures.keys],
  };
}

export function requestOf(form: PolicyForm, creating: boolean): PolicyRequest {
  const body: PolicyRequest = {
    notes: form.notes,
    vulnerabilities: {
      mode: form.vulnerabilityMode,
      severity: severityOf(form.severity),
      malicious: form.malicious,
      unscanned: form.unscanned,
      notify: severityOf(form.notify),
    },
    licenses: {
      mode: form.licenseMode,
      list: listOf(form.licenseList),
      ids: idsOf(form.licenseIds),
      unknown: form.unknownLicense,
    },
    signatures: {
      mode: form.signatureMode,
      keys: form.signatureKeys,
    },
  };
  if (creating) {
    body.name = form.name;
  }
  return body;
}

export function blocksEntered(saved: PolicyForm, next: PolicyForm): BlockName[] {
  const entered: BlockName[] = [];
  if (next.vulnerabilityMode === 'block' && saved.vulnerabilityMode !== 'block') {
    entered.push('vulnerabilities');
  }
  if (next.licenseMode === 'block' && saved.licenseMode !== 'block') {
    entered.push('licenses');
  }
  if (next.signatureMode === 'block' && saved.signatureMode !== 'block') {
    entered.push('signatures');
  }
  return entered;
}
