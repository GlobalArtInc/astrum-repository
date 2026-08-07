

import React, { createContext, useContext, useRef, useState, useEffect } from 'react';
import { Tooltip as RadixTooltip } from '@radix-ui/themes';
import { Provider as RadixTooltipProvider } from '@radix-ui/react-tooltip';

const TooltipContainerContext = createContext(null);

/**
 * Provider that renders a portal container div inside the Radix Theme subtree.
 * Tooltips rendered to this container inherit theme variables (--gray-12, --gray-1)
 * instead of losing them when portaled to body.
 */
export function TooltipContainerProvider({ children }) {
  const containerRef = useRef(null);
  const [container, setContainer] = useState(null);

  useEffect(() => {
    setContainer(containerRef.current);
  }, []);

  return (
    <RadixTooltipProvider delayDuration={200}>
      <TooltipContainerContext.Provider value={container}>
        <div
          ref={containerRef}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: 0,
            height: 0,
            overflow: 'visible',
            pointerEvents: 'none',
            zIndex: 10000,
          }}
          aria-hidden
        />
        {children}
      </TooltipContainerContext.Provider>
    </RadixTooltipProvider>
  );
}

/**
 * Hook to get the portal container. Use for DropdownMenu.Content, Select.Content, etc.
 * so they render above the top nav (same z-index layer as tooltips).
 */
export function usePortalContainer() {
  return useContext(TooltipContainerContext);
}

/**
 * Tooltip that renders inside the Theme subtree so it inherits Radix theme variables.
 * Falls back to body when used outside TooltipContainerProvider.
 */
export function Tooltip(props) {
  const container = useContext(TooltipContainerContext);
  return <RadixTooltip container={container} {...props} />;
}
