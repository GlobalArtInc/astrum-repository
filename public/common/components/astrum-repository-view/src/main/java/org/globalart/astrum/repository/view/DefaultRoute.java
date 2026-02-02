
package org.globalart.astrum.repository.view;

import java.util.List;

import org.globalart.astrum.repository.view.matchers.AlwaysMatcher;

/**
 * View default route.
 *
 * @since 3.0
 */
public class DefaultRoute
    extends Route
{
  private static final Matcher MATCHER = new AlwaysMatcher();

  public DefaultRoute(final List<Handler> handlers) {
    super(MATCHER, handlers);
  }
}
