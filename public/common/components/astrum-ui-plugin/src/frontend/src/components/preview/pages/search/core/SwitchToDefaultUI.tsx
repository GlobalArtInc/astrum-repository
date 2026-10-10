

import React from 'react';
import { buildDefaultSearchRoute } from './search.routes';

/**
 * SwitchToDefaultUI - Link to switch from Preview UI to Default UI
 * 
 * This component provides a way for users to return to the ExtJS-based
 * Default UI search when they're in the Preview UI GA search.
 * 
 * Owner: Agent 0 (Tech Lead)
 */

interface SwitchToDefaultUIProps {
  /** Optional custom label */
  label?: string;
  /** Optional CSS class */
  className?: string;
}

export function SwitchToDefaultUI({ 
  label = 'Switch to Classic Search',
  className = ''
}: SwitchToDefaultUIProps) {
  const defaultUrl = buildDefaultSearchRoute();
  
  return (
    <a 
      href={defaultUrl}
      className={`as-switch-to-default-ui ${className}`.trim()}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.5rem',
        fontSize: '0.875rem',
        color: 'var(--as-text-color-secondary, #6b7280)',
        textDecoration: 'none',
      }}
    >
      <svg 
        width="16" 
        height="16" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        strokeWidth="2"
        strokeLinecap="round" 
        strokeLinejoin="round"
      >
        <path d="M19 12H5M12 19l-7-7 7-7" />
      </svg>
      {label}
    </a>
  );
}

export default SwitchToDefaultUI;


