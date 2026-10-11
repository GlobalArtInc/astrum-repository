

import React, {useState} from 'react';
import {DropdownMenu, IconButton} from '@radix-ui/themes';
import {Tooltip, usePortalContainer, ExtJS} from '@globalart/astrum-ui-plugin';
import {Languages} from 'lucide-react';
import UIStrings from '../../constants/UIStrings';

export const LANGUAGE_COOKIE = 'astrum-language';

const ONE_YEAR_IN_SECONDS = 365 * 24 * 60 * 60;

function readUiLanguage() {
  try {
    return ExtJS.state()?.getValue('uiLanguage') || null;
  }
  catch (e) {
    return null;
  }
}

/**
 * Remembers the chosen language in a cookie and reloads the page, which the server then renders in that language.
 */
export function chooseLanguage(language) {
  document.cookie = `${LANGUAGE_COOKIE}=${encodeURIComponent(language)}; path=/; max-age=${ONE_YEAR_IN_SECONDS}; SameSite=Lax`;
  window.location.reload();
}

/**
 * LanguageMenuRadix - lets the user pick the language of the UI among the ones the server has translations for.
 * Hidden while there is only one.
 */
export default function LanguageMenuRadix() {
  const portalContainer = usePortalContainer();
  // The language only changes with a page reload, so it is read once
  const [uiLanguage] = useState(readUiLanguage);
  const languages = uiLanguage?.available || [];

  if (languages.length < 2) {
    return null;
  }

  return (
    <DropdownMenu.Root>
      <Tooltip content={UIStrings.SHELL.HEADER.LANGUAGE}>
        <DropdownMenu.Trigger>
          <IconButton
            variant="outline"
            size="3"
            color="gray"
            className="as-global-header-radix__icon-button"
            aria-label={UIStrings.SHELL.HEADER.LANGUAGE}
            data-analytics-id="astrum-header-language"
          >
            <Languages size={18} aria-hidden />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>

      <DropdownMenu.Content
        align="end"
        container={portalContainer}
        color="gray"
        variant="soft"
        className="as-global-header-radix__menu"
      >
        <DropdownMenu.RadioGroup
          value={uiLanguage.current}
          onValueChange={(language) => language !== uiLanguage.current && chooseLanguage(language)}
        >
          {languages.map(({id, name}) => (
            <DropdownMenu.RadioItem key={id} value={id} lang={id}>
              {name}
            </DropdownMenu.RadioItem>
          ))}
        </DropdownMenu.RadioGroup>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
