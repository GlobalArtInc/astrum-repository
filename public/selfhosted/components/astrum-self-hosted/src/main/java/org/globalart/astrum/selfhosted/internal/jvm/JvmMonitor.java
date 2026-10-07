
package org.globalart.astrum.selfhosted.internal.jvm;

/**
 * Interface for JVM monitoring implementations.
 *
 * Implementations monitor specific aspects of JVM health (GC, memory, etc.)
 * and log warnings when thresholds are exceeded.
 */
public interface JvmMonitor
{
  /**
   * Perform monitoring check and log if thresholds are exceeded.
   */
  void check();

  /**
   * Initialize monitoring state (called once on service start).
   */
  default void initialize() {
    // Optional initialization
  }

  /**
   * Cleanup monitoring state (called once on service stop).
   */
  default void cleanup() {
    // Optional cleanup
  }
}
