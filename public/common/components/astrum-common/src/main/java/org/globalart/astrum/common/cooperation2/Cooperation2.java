
package org.globalart.astrum.common.cooperation2;

import java.io.IOException;
import java.util.Map;

/**
 * @since 3.41
 */
public interface Cooperation2
{
  /**
   * Sets the function that is used to perform the work. Depending on the thread, or the node chosen to perform the
   * co-operation there is no guarantee that the function will be called and the {@link Cooperation2} built may return
   * the result from another thread.
   *
   * @param workFunction a function which can be used to perform the operation
   * @param <RET> the type of the return value
   * @return the resulting builder
   */
  <RET> Builder<RET> on(IOCall<RET> workFunction);

  /**
   * @return number of threads cooperating per request-key.
   */
  Map<String, Integer> getThreadCountPerKey();

  interface Builder<RET>
  {
    /**
     * Set an {@link IOCheck} that can be used to verify that the work of this cooperation has already been performed.
     * If this check is not provided it is assumed the work is not complete.
     *
     * This function is not guaranteed to be called (if another thread, or node is the lead).
     *
     * Exceptions thrown by this supplier will terminate the co-operation.
     *
     * @param checkFunction a supplier which can be used to determine whether the work required for a co-operation key
     *          has been completed.
     * @return the resulting builder
     */
    Builder<RET> checkFunction(IOCheck<RET> checkFunction);

    /**
     * The co-operation may (depending on implementation) perform the work if concurrency controls timeout.
     */
    Builder<RET> performWorkOnFail(final boolean performWorkOnFail);

    /**
     * Perform the co-operation. (Note implementations may not execute the work asynchronously)
     */
    RET cooperate(String action, String... scopes) throws IOException;
  }
}
