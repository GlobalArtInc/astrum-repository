import { V1 } from '../../../api';

export const LICENCES = `${V1}/security/licenses`;
export const LICENCE_EXPORT = `${LICENCES}/export.csv`;

export const LICENCE_STATES = ['spdx', 'mapped', 'unknown', 'none'] as const;
export type LicenceState = (typeof LICENCE_STATES)[number];

export const LICENCE_VERDICTS = ['pass', 'blocked', 'reported', 'unjudged'] as const;
export type LicenceVerdict = (typeof LICENCE_VERDICTS)[number];

export interface LicenceRow {
  repository: string;
  format: string;
  namespace: string;
  name: string;
  version: string;
  declared: string;
  expression: string | null;
  ids: string[];
  state: LicenceState;
  verdict: LicenceVerdict;
}

export interface LicenceFilters {
  repository: string;
  license: string;
  state: string;
  format: string;
}
