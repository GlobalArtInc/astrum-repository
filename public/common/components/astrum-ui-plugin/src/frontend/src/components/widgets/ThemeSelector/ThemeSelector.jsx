
import React, {useEffect, useState} from 'react';

import {NxButton, NxFontAwesomeIcon} from "@globalart/react-shared-components";
import {faMoon, faSun} from "@fortawesome/free-solid-svg-icons";

export function ThemeSelector() {
  const [theme, setTheme] = useState(localStorage.getItem('theme') || 'light');
  const themeIcon = theme === 'dark' ? faMoon : faSun;
  const themeTitle = theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode';

  useEffect(() => {
    document.documentElement.classList.add('nx-html--enable-color-schemes');
    if (theme === 'dark') {
      document.documentElement.classList.add('nx-html--dark-mode');
      document.documentElement.classList.remove('nx-html--light-mode');
    } else {
      document.documentElement.classList.add('nx-html--light-mode');
      document.documentElement.classList.remove('nx-html--dark-mode');
    }
  }, [theme]);

  function toggleTheme() {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
  }

  return (
      <NxButton variant="icon-only" title={themeTitle} onClick={toggleTheme}>
        <NxFontAwesomeIcon icon={themeIcon}/>
      </NxButton>
  );
}
