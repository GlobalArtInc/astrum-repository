
import React from 'react';
import { createRoot } from 'react-dom/client';

/**
 * Exposes React and createRoot on window so that ExtJS views can mount React
 * components. AS/view/SecondaryContainer.js relies on both:
 *   - window.react  for react.createElement()
 *   - window.createRoot  for mounting React trees into ExtJS DOM nodes
 *
 * Must be called from each implementation plugin's entry point (astrum-coreui-plugin,
 * astrum-cloudui-plugin). Placing it here rather than in astrum-rapture avoids
 * bundling react and react-dom twice — the plugin already needs them for its
 * own rendering.
 */
export default function exposeCreateRoot() {
  window.react = React;
  window.createRoot = createRoot;
}
