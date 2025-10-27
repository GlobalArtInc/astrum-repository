
package org.globalart.astrum.common.lifecycle;

/**
 * Component life-cycle.
 */
public interface Lifecycle
{
  void start() throws Exception;

  void stop() throws Exception;
}
