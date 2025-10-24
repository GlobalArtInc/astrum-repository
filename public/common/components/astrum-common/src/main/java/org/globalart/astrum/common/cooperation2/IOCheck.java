
package org.globalart.astrum.common.cooperation2;

import java.io.IOException;
import java.util.Optional;

/**
 * A function used during cooperation to check whether 'work' has been done.
 *
 * @since 3.41
 */
@FunctionalInterface
public interface IOCheck<T>
{
  /**
   * @return a non-empty Optional containing the result of the cooperation, or an empty Optional if the work is
   *         incomplete
   * @throws IOException
   */
  Optional<T> check() throws IOException;
}
