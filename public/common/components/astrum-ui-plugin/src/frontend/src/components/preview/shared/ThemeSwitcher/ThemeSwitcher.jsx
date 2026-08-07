

import React from 'react';
import { IconButton, Tooltip } from '@radix-ui/themes';
import { Sun, MoonStar } from 'lucide-react';
import { useTheme, THEMES } from '../../../../contexts/ThemeContext';

/**
 * ThemeSwitcher - Icon-only toggle for dark/light mode.
 * Uses Radix IconButton with Lucide icons (16px).
 * Toggles between Light and Dark only.
 */
export function ThemeSwitcher() {
  const { effectiveTheme, setTheme } = useTheme();

  const toggleTheme = () => {
    setTheme(effectiveTheme === 'dark' ? THEMES.LIGHT : THEMES.DARK);
  };

  const getIcon = () => {
    return effectiveTheme === 'dark' ? <Sun size={16} /> : <MoonStar size={16} />;
  };

  const getTooltip = () => {
    return effectiveTheme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode';
  };

  return (
    <Tooltip content={getTooltip()}>
      <IconButton
        variant="outline"
        size="2"
        color="gray"
        aria-label={getTooltip()}
        onClick={toggleTheme}
      >
        {getIcon()}
      </IconButton>
    </Tooltip>
  );
}

export default ThemeSwitcher;
