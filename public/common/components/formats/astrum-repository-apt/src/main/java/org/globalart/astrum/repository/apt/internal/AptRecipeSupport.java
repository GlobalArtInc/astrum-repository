
package org.globalart.astrum.repository.apt.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.config.Configuration;

/**
 * Support for Apt recipes
 *
 * @since 3.17
 */
public abstract class AptRecipeSupport
    extends RecipeSupport
{
  static {
    Configuration.addSensitiveFieldName("aptSigning");
  }

  protected AptRecipeSupport(
      final Type type,
      final Format format)
  {
    super(type, format);
  }
}
