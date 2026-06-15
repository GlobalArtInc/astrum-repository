

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
 * Only the latin subset is imported, for the weights the design system uses. Fontsource's
 * per-subset stylesheets carry no unicode-range descriptor, so importing more than one
 * subset for the same family and weight would make the later @font-face shadow the earlier
 * one entirely rather than combining them. Glyphs outside the latin subset fall back
 * through the rest of the font stack, which is the intended behaviour.
 */

// Inter — UI text. Weights 400/500/600/700.
import '@fontsource/inter/latin-400.css';
import '@fontsource/inter/latin-500.css';
import '@fontsource/inter/latin-600.css';
import '@fontsource/inter/latin-700.css';

// Geist Mono — code, logs, and other monospaced content. Weight 400.
import '@fontsource/geist-mono/latin-400.css';
