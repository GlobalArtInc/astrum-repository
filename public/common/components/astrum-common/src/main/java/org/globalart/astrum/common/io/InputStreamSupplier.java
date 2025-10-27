
package org.globalart.astrum.common.io;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nonnull;

/**
 * For use instead of Supplier<InputStream> to reduce errors which leave streams open.
 *
 * @since 3.31
 */
@FunctionalInterface
public interface InputStreamSupplier
{
  @Nonnull
  InputStream get() throws IOException;
}
