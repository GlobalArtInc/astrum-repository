

import ExtJS from '../interface/ExtJS';
import { isVisible } from '../interface/NavigationUtils';

export default function useIsVisible(visibilityRequirements) {
  return ExtJS.useVisiblityWithChanges(() => isVisible(visibilityRequirements));
}
