

import { ExtJS } from '../../../../interface/ExtJS';

/** Reads the `isCloud` flag from ExtJS global state. Session-scoped (value is fixed for the page lifecycle). */
export function useIsCloud(): boolean {
  return ExtJS.useState?.(() => ExtJS.state()?.getValue?.('isCloud', false)) ?? false;
}
