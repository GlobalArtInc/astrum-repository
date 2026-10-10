import React from 'react';
import {render, screen, fireEvent} from '@testing-library/react';
import {Theme} from '@radix-ui/themes';
import '@testing-library/jest-dom';

let mockUiLanguage;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ExtJS: {
    state: () => ({getValue: (key) => (key === 'uiLanguage' ? mockUiLanguage : undefined)}),
  },
  Tooltip: ({children}) => children,
  usePortalContainer: () => undefined,
}));

import LanguageMenuRadix, {LANGUAGE_COOKIE} from '../LanguageMenuRadix';

const BOTH = {
  current: 'en',
  available: [{id: 'en', name: 'English'}, {id: 'ru', name: 'Русский'}],
};

function renderMenu() {
  return render(
    <Theme>
      <LanguageMenuRadix />
    </Theme>
  );
}

// Radix opens its menus on pointer events jsdom lacks, so the menu is opened from the keyboard
function openMenu() {
  fireEvent.keyDown(screen.getByRole('button', {name: 'Language'}), {key: 'Enter'});
}

describe('LanguageMenuRadix', () => {
  const location = window.location;
  let reload;

  beforeEach(() => {
    reload = jest.fn();
    delete window.location;
    window.location = {...location, reload};
    document.cookie = `${LANGUAGE_COOKIE}=; path=/; max-age=0`;
  });

  afterEach(() => {
    window.location = location;
  });

  it('is hidden while the server has no translations', () => {
    mockUiLanguage = {current: 'en', available: [{id: 'en', name: 'English'}]};

    renderMenu();

    expect(screen.queryByRole('button', {name: 'Language'})).not.toBeInTheDocument();
  });

  it('is hidden when the state carries no languages', () => {
    mockUiLanguage = undefined;

    renderMenu();

    expect(screen.queryByRole('button', {name: 'Language'})).not.toBeInTheDocument();
  });

  it('lists the languages and marks the current one', () => {
    mockUiLanguage = BOTH;

    renderMenu();
    openMenu();

    expect(screen.getByRole('menuitemradio', {name: 'English'})).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('menuitemradio', {name: 'Русский'})).toHaveAttribute('aria-checked', 'false');
  });

  it('remembers the chosen language and reloads the page', () => {
    mockUiLanguage = BOTH;

    renderMenu();
    openMenu();
    fireEvent.click(screen.getByRole('menuitemradio', {name: 'Русский'}));

    expect(document.cookie).toContain(`${LANGUAGE_COOKIE}=ru`);
    expect(reload).toHaveBeenCalledTimes(1);
  });

  it('does nothing when the current language is chosen again', () => {
    mockUiLanguage = BOTH;

    renderMenu();
    openMenu();
    fireEvent.click(screen.getByRole('menuitemradio', {name: 'English'}));

    expect(document.cookie).not.toContain(`${LANGUAGE_COOKIE}=`);
    expect(reload).not.toHaveBeenCalled();
  });
});
