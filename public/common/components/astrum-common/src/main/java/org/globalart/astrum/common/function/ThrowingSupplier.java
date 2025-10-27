
package org.globalart.astrum.common.function;

import java.io.IOException;

/**
 * @since 3.41
 */
@FunctionalInterface
public interface ThrowingSupplier<T>
{
  T get() throws IOException;
}
