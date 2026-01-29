
package org.globalart.astrum.repository.recipe;

/**
 * Builder of router configurations.
 *
 * This interface helps bridge to the view API which now lives in another module.
 *
 * @since 3.22
 */
public interface RouterBuilder
{
  RouterBuilder route(Route route);
}
