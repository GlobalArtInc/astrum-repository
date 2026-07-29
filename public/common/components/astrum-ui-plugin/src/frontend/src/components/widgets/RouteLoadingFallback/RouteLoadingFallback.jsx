
import React, { useState, useEffect } from 'react';
import { NxPageMain, NxLoadingSpinner } from '@globalart/react-shared-components';

import './RouteLoadingFallback.scss';

/**
 * Loading fallback component displayed while route chunks are loading.
 *
 * Uses a delayed display strategy to prevent "flash of loading" on fast route transitions:
 * - For loads < 300ms: Shows nothing (feels instant)
 * - For loads > 300ms: Shows spinner (provides feedback)
 *
 * This creates a better perceived performance by avoiding jarring spinner flashes
 * on quick navigation while still providing feedback for genuinely slow loads.
 */
export default function RouteLoadingFallback() {
  const [show, setShow] = useState(false);

  useEffect(() => {
    // Delay showing the spinner to avoid flash on fast loads
    const timer = setTimeout(() => {
      setShow(true);
    }, 300);

    // Cleanup: Cancel timer if component unmounts before 300ms
    return () => {
      clearTimeout(timer);
    };
  }, []); // Empty deps - only run once on mount

  // Show nothing during the initial 300ms delay
  if (!show) {
    return null;
  }

  // After 300ms, show the centered loading spinner
  return (
    <NxPageMain>
      <div className="as-route-loading">
        <NxLoadingSpinner />
      </div>
    </NxPageMain>
  );
}
