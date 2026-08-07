
import { createContext } from 'react';

/**
 * Context for Preview UI mode.
 * Components can use this to determine if they should use preview.* route variants.
 */
export const PreviewUIContext = createContext(false);

PreviewUIContext.displayName = 'PreviewUIContext';
