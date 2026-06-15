
import { lazy } from 'react';
import { MissingRoutePage } from '@globalart/astrum-ui-plugin';

/**
 * Creates a lazy-loaded component with proper chunk naming for debugging.
 *
 * This utility wraps React.lazy() to provide:
 * - Dynamic imports with webpack magic comments for chunk naming
 * - Error handling for failed chunk loads with redirect to 404 page
 * - Support for both default and named exports
 *
 * IMPORTANT: Include webpack magic comments inline for proper chunk naming.
 * Webpack performs static analysis at build time and needs to see the comment
 * directly in the import() call.
 *
 * @param {Function} importFn - A function that returns a dynamic import promise.
 *                               Must include webpackChunkName magic comment for proper naming.
 * @returns {React.LazyExoticComponent} A lazy-loaded React component
 *
 * @example
 * // Standard usage with webpack magic comment for chunk naming
 * const Welcome = lazyLoad(
 *   () => import(/* webpackChunkName: "browse-Welcome" *\/ '../../components/pages/user/Welcome/Welcome')
 * );
 *
 * @example
 * // For a component with named export, convert to default
 * const Settings = lazyLoad(
 *   () => import(/* webpackChunkName: "admin-Settings" *\/ '../../components/pages/admin/Settings')
 *     .then(module => ({ default: module.Settings }))
 * );
 */
export function lazyLoad(importFn) {
  return lazy(() =>
    importFn()
      .then((module) => {
        // Handle both default and named exports
        if (module.default) {
          return module;
        }
        // If no default export, throw an error with helpful message
        throw new Error(
          'Module loaded by lazyLoad does not have a default export. Please check the component is exported as default or wrap it: .then(m => ({ default: m.ComponentName }))'
        );
      })
      .catch((error) => {
        console.error('Failed to load lazy-loaded chunk:', error);
        // Return 404 page instead of throwing to prevent error boundary
        return { default: MissingRoutePage };
      })
  );
}
