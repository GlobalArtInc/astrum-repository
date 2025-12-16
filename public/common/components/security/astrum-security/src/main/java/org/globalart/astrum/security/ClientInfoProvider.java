
package org.globalart.astrum.security;

import javax.annotation.Nullable;

/**
 * Manages and provides {@link ClientInfo} instances.
 *
 * @author cstamas
 * @since 2.1
 */
public interface ClientInfoProvider
{
  /**
   * Returns the {@link ClientInfo} for current thread. It will be non-null if this thread is a REST (or better HTTP)
   * Request processing thread, and {@code null} if this is a non REST Request processing thread (like a scheduled task
   * threads are).
   *
   * @return the current thread's {@link ClientInfo} or {@code null} if none available.
   */
  @Nullable
  ClientInfo getCurrentThreadClientInfo();

  /**
   * Set the custom IP address and the user id of the client.
   * This info is stored on the thread and must be unset using {@link #unsetClientInfo()}
   * to prevent leakage.
   *
   * @param remoteIp the IP address of the client that sent the request.
   * @param userId the user id.
   */
  void setClientInfo(String remoteIp, String userId);

  /**
   * Unset the custom IP address and the user id of the client that was previously set using
   * {@link #setClientInfo(String, String)}.
   * This is necessary to prevent leakage of client information across different threads.
   */
  void unsetClientInfo();
}
