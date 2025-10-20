
package org.globalart.goodies.packageurl;

/**
 * Thrown when package-url component that is required is missing.
 *
 * @since 1.1.0
 */
public class MissingComponentException
    extends InvalidException
{
  private final String name;

  MissingComponentException(final String name) {
    super("Missing required component: " + name);
    this.name = name;
  }

  public String getName() {
    return name;
  }
}
