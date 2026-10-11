
import { createContext } from 'react';

/**
 * Context for Preview UI mode.
 * Components can use this to determine if they should use preview.* route variants.
 *
 * Astrum One is the only UI, so it defaults to true outside a provider as well.
 */
export const PreviewUIContext = createContext(true);

PreviewUIContext.displayName = 'PreviewUIContext';
