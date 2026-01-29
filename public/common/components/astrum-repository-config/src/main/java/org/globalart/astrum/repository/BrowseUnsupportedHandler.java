
package org.globalart.astrum.repository;

import org.globalart.astrum.repository.recipe.Handler;
import org.globalart.astrum.repository.recipe.Route;

/**
 * Handles routes that don't support browse.
 *
 * This interface helps bridge to the real implementation which now lives in another module.
 *
 * @since 3.22
 */
public interface BrowseUnsupportedHandler
    extends Handler
{
  Route getRoute();
}
