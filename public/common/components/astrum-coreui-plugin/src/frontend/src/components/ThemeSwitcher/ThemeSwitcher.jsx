import React from 'react';
import classNames from 'classnames';
import { IconButton } from '@radix-ui/themes';
import { Tooltip } from '@globalart/astrum-ui-plugin';
import { Sun, Moon } from 'lucide-react';
import { useTheme, THEMES } from '../../contexts/ThemeContext';
import UIStrings from '../../constants/UIStrings';

const {THEME} = UIStrings.SHELL;

/**
 * ThemeSwitcher - Icon-only toggle for dark/light mode.
 * Shows the sun in the dark theme and the moon in the light one, i.e. the theme a click switches to.
 */
export function ThemeSwitcher({className}) {
  const { effectiveTheme, setTheme } = useTheme();
  const isDark = effectiveTheme === THEMES.DARK;
  const label = isDark ? THEME.SWITCH_TO_LIGHT : THEME.SWITCH_TO_DARK;

  const toggleTheme = () => {
    setTheme(isDark ? THEMES.LIGHT : THEMES.DARK);
  };

  return (
    <Tooltip content={label}>
      <IconButton
        variant="outline"
        size="3"
        color="gray"
        className={classNames('theme-switcher', className)}
        aria-label={label}
        data-analytics-id="as-header-toggle-theme"
        onClick={toggleTheme}
      >
        {isDark ? <Sun size={18} aria-hidden /> : <Moon size={18} aria-hidden />}
      </IconButton>
    </Tooltip>
  );
}

export default ThemeSwitcher;
