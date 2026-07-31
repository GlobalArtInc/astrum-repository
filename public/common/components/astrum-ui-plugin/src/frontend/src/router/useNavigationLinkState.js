
import { useIsActive, useRouter, useSref } from '@uirouter/react';
import { useIsVisible } from '../interface/NavigationUtils';

/**
 * @param name - route sate name
 * @param selectedState - optional - the state to use as the selected state, if different from the name parameter
 * @param params - optional - url parameters
 */
export function useNavigationLinkState(name, selectedState, params) {
  const router = useRouter();
  const state = router.stateRegistry.get(name);

  // If the route doesn't exist, hide the menu item and log an error
  if (!state) {
    console.error(`LeftNavigationMenuItem -> route "${name}" does not exist. Menu item will be hidden.`);
    return { isVisible: false, href: null, isSelected: false, icon: null };
  }

  const data = state?.data || {};
  const { text, icon, visibilityRequirements } = data;

  const isSelected = useIsActive(selectedState || name);

  const { href } = useSref(name, params);

  const isVisible = useIsVisible(visibilityRequirements);

  return { isVisible, href, isSelected, text, icon };
}
