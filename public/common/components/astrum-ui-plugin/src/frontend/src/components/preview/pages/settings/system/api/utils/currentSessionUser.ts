

import { ExtJS } from '../../../../../../../interface/ExtJS';

export function readCurrentSessionUserId(): string | null {
  try {
    const u = ExtJS.state()?.getValue?.('user') as { id?: string } | undefined;
    const id = u?.id;
    return typeof id === 'string' && id.length > 0 ? id : null;
  } catch {
    return null;
  }
}
