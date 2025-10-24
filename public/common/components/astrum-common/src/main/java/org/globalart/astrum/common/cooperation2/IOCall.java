
package org.globalart.astrum.common.cooperation2;

import java.io.IOException;

/**
 * A function encapsulating the work to be done by a Cooperation.
 *
 * @since 3.41
 */
@FunctionalInterface
public interface IOCall<T>
{
  T call() throws IOException;
}
