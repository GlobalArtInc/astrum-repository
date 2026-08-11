

import { ExtJS } from '../../../../interface/ExtJS';

export function useIsClustered(): boolean {
  return ExtJS.useState?.(() => ExtJS.state()?.getValue?.('astrum.datastore.clustered.enabled', false)) ?? false;
}
