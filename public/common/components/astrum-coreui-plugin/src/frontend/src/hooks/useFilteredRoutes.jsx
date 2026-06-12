

import { useRouter } from '@uirouter/react';
import { ExtJS } from '@globalart/astrum-ui-plugin';

export default function useFilteredRoutes(filter) {
  const router = useRouter();

  return ExtJS.useVisiblityWithChanges(() => {
    const allStates = router.stateRegistry.get();

    return allStates.filter(filter);
  });
}
