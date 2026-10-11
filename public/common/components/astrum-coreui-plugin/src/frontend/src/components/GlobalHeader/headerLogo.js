import proLogo from '../../../../art/logos/logo-pro-edition-header.svg';
import proDarkLogo from '../../../../art/logos/logo-pro-edition-header-dark.svg';
import ceLogo from '../../../../art/logos/logo-community-edition-header.svg';
import ceDarkLogo from '../../../../art/logos/logo-community-edition-header-dark.svg';
import coreLogo from '../../../../art/logos/logo-core-edition-header.svg';
import coreDarkLogo from '../../../../art/logos/logo-core-edition-header-dark.svg';
import markLogo from '../../../../art/logos/astrum-mark.svg';

/**
 * The logo of the header for the edition and theme: the mark with the product name.
 */
export function getHeaderLogo(edition, isDark) {
  if (edition === 'COMMUNITY') {
    return isDark ? ceDarkLogo : ceLogo;
  }
  if (edition === 'PRO') {
    return isDark ? proDarkLogo : proLogo;
  }
  return isDark ? coreDarkLogo : coreLogo;
}

/**
 * The mark alone, for headers too narrow for the name (phones).
 */
export const HEADER_MARK = markLogo;
