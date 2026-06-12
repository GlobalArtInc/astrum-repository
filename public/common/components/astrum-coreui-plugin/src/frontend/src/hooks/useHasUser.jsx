

import { ExtJS } from '@globalart/astrum-ui-plugin';

export default function useHasUser() {
  const user = ExtJS.useUser();

  return !!user;
}
