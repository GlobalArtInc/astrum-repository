
package org.globalart.astrum.rest.client.internal;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * ClassLoader which bridges two classloaders into a single classloader view.
 *
 * @since 3.2.1
 */
class BridgeClassLoader
    extends ClassLoader
{
  private final ClassLoader secondary;

  public BridgeClassLoader(final ClassLoader primary, final ClassLoader secondary) {
    super(checkNotNull(primary));
    this.secondary = checkNotNull(secondary);
  }

  @Override
  protected Class<?> findClass(final String name) throws ClassNotFoundException {
    return secondary.loadClass(name);
  }
}
