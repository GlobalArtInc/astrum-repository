

/**
 * Self-hosted webfont registration for Super UI (ASTRUM-52338).
 *
 * These imports only register @font-face rules. The font-family that actually applies
 * them is scoped to `.radix-themes` in styles/theme-variables.css so Classic UI (ExtJS)
 * typography is unaffected.
 *
 * Imported from JavaScript rather than via a Sass `@import` in App.scss on purpose: Sass
 * inlines an @import into the importing stylesheet, which would make these rules part of
 * App.scss's own module and subject to that module's `url: false` css-loader option. Kept
 * as JS imports, each file stays its own module and matches the @fontsource-scoped
 * css-loader branch in rspack.common.js, so the relative url(./files/*.woff2) references
 * resolve and the font files are emitted.
 *
 * The full per-weight stylesheets are imported: each declares one @font-face per subset with
 * its unicode-range, so the browser downloads only the subsets a page uses and Cyrillic text is
 * set in the same fonts as Latin text.
 */

// Geist — UI text. Weights 400/500/600/700.
import '@fontsource/geist/400.css';
import '@fontsource/geist/500.css';
import '@fontsource/geist/600.css';
import '@fontsource/geist/700.css';

// Geist Mono — code, logs, and other monospaced content. Weights 400/500.
import '@fontsource/geist-mono/400.css';
import '@fontsource/geist-mono/500.css';
