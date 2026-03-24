
package org.globalart.astrum.repository.maven.internal.filter;

import java.io.Closeable;
import java.io.IOException;

import com.google.common.base.Predicate;

/**
 * Strategy for filtering Maven records
 *
 * @since 3.11
 */
public interface DuplicateDetectionStrategy<T>
    extends Predicate<T>, Closeable
{
  @Override
  default void close() throws IOException {
    // no-op
  }
}
