
import React, {useEffect, useState} from 'react';

import {AsButton, AsFontAwesomeIcon} from "@globalart/react-shared-components";
import {faMoon, faSun} from "@fortawesome/free-solid-svg-icons";

export function ThemeSelector() {
  const [theme, setTheme] = useState(localStorage.getItem('theme') || 'light');
  const themeIcon = theme === 'dark' ? faMoon : faSun;
  const themeTitle = theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode';

  useEffect(() => {
    document.documentElement.classList.add('as-html--enable-color-schemes');
    if (theme === 'dark') {
      document.documentElement.classList.add('as-html--dark-mode');
      document.documentElement.classList.remove('as-html--light-mode');
    } else {
      document.documentElement.classList.add('as-html--light-mode');
      document.documentElement.classList.remove('as-html--dark-mode');
    }
  }, [theme]);

  function toggleTheme() {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
  }

  return (
      <AsButton variant="icon-only" title={themeTitle} onClick={toggleTheme}>
        <AsFontAwesomeIcon icon={themeIcon}/>
      </AsButton>
  );
}
