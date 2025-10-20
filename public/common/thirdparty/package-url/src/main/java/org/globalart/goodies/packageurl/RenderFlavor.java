
package org.globalart.goodies.packageurl;

import javax.annotation.Nullable;

/**
 * How to render the Package-URL.
 *
 * @since 1.1.0
 */
public enum RenderFlavor
{
  /**
   * Render with {@link PackageUrl#SCHEME} (default).
   */
  SCHEME,

  /**
   * Render with-out {@link PackageUrl#SCHEME}.
   */
  SCHEMELESS;

  private static volatile RenderFlavor _default = SCHEME;

  public static RenderFlavor getDefault() {
    return _default;
  }

  /**
   * Set the default flavor, or null for the original default.
   */
  public static void setDefault(@Nullable final RenderFlavor flavor) {
    _default = flavor == null ? SCHEME : flavor;
  }
}
