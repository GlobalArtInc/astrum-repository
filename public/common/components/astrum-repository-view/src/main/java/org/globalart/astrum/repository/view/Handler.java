
package org.globalart.astrum.repository.view;

import javax.annotation.Nonnull;

/**
 * View handler.
 *
 * @since 3.0
 */
public interface Handler
    extends org.globalart.astrum.repository.recipe.Handler
{
  @Nonnull
  Response handle(@Nonnull Context context) throws Exception;
}
