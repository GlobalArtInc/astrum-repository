

import React from "react";
import { ExtJS } from '../../interface/ExtJS';

import './LoginLayout.scss';

// Edition constants - must match backend values
const EDITION_COMMUNITY = "COMMUNITY";
const EDITION_PRO = "PRO";

/**
 * Minimal layout for login page with only branding/logo in header.
 * No navigation menu, theme switcher, or other header components.
 * Always uses light theme for login page.
 */
export default function LoginLayout({ children, logoConfig }) {

  const edition = ExtJS.useState(() => ExtJS.state().getEdition());
  const contextPath = ExtJS.useState(() => ExtJS.state().getValue('astrum-context-path', ''));

  function getLogo() {
    // Light-only theme, so we only use light logos
    return edition === EDITION_COMMUNITY ? (logoConfig?.ceLight || logoConfig?.proLight)
        : edition === EDITION_PRO ? logoConfig?.proLight
        : (logoConfig?.coreLight || logoConfig?.proLight); // Core or fallback to pro
  }


  const homeHash = '#browse/welcome';
  const homeHref = contextPath ? `${contextPath}/${homeHash}` : `/${homeHash}`;

  const handleLogoClick = (e) => {
    e.preventDefault();
    window.location.hash = homeHash;
  };

  return (
    <div className="as-login-layout">
      <header className="as-login-header" role="banner">
        <a
          href={homeHref}
          className="as-login-header__link"
          title="Home"
          aria-label="Astrum Repository home"
          onClick={handleLogoClick}
        >
          <img
            src={getLogo()}
            alt="Astrum Repository"
            className="as-login-header__logo"
            width={220}
            height={32}
            decoding="async"
          />
        </a>
      </header>
      <main className="as-login-main">
        {children}
      </main>
    </div>
  );
}
