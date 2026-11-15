import { V1 } from '../../../api';

export const SHARE_LINKS = `${V1}/share-links`;
export const SHARE_CREATE = 'astrum:sharelinks:create';
export const SHARE_READ = 'astrum:sharelinks:read';
export const SHARE_DELETE = 'astrum:sharelinks:delete';

export interface ShareLink {
  id: string;
  repository: string;
  path: string;
  prefix: boolean;
  note: string;
  createdBy: string;
  createdAt: string;
  expiresAt: string;
  maxDownloads: number | null;
  downloads: number;
  bytesServed: number;
  lastUsedAt: string | null;
  expired: boolean;
}

export interface CreatedShareLink extends ShareLink {
  url: string;
}

export interface ShareTarget {
  repository: string;
  path: string;
  prefix: boolean;
}

export interface NewShareLink extends ShareTarget {
  expiresInHours: number;
  maxDownloads?: number;
  note?: string;
}

export type Lifetime = '1h' | '1d' | '7d' | 'custom';

export interface LifetimePreset {
  id: Exclude<Lifetime, 'custom'>;
  hours: number;
}

export const LIFETIME_PRESETS: LifetimePreset[] = [
  { id: '1h', hours: 1 },
  { id: '1d', hours: 24 },
  { id: '7d', hours: 168 },
];

export interface ShareValues {
  lifetime: Lifetime;
  hours: number | null;
  maxDownloads: number | null;
  note: string;
}

export const INITIAL_SHARE_VALUES: ShareValues = {
  lifetime: '7d',
  hours: 24,
  maxDownloads: null,
  note: '',
};

export function lifetimeHours(values: ShareValues): number {
  if (values.lifetime === 'custom') {
    return values.hours ?? 0;
  }
  return LIFETIME_PRESETS.find((preset) => preset.id === values.lifetime)?.hours ?? 0;
}

export function shareRequest(target: ShareTarget, values: ShareValues): NewShareLink {
  const request: NewShareLink = { ...target, expiresInHours: lifetimeHours(values) };
  if (!target.prefix && values.maxDownloads !== null) {
    request.maxDownloads = values.maxDownloads;
  }
  const note = values.note.trim();
  if (note !== '') {
    request.note = note;
  }
  return request;
}
