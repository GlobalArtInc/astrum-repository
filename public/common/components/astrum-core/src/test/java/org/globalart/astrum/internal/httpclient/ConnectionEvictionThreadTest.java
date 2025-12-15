
package org.globalart.astrum.internal.httpclient;

import java.util.concurrent.TimeUnit;

import org.apache.http.conn.HttpClientConnectionManager;
import org.junit.Test;

import static org.mockito.Mockito.atLeast;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

/**
 * Tests for {@link ConnectionEvictionThread}.
 */
public class ConnectionEvictionThreadTest
{
  /**
   * Verify that ClientConnectionManager are called.
   */
  @Test
  public void connectionEvictedIn5Seconds() throws Exception {
    final HttpClientConnectionManager clientConnectionManager = mock(HttpClientConnectionManager.class);

    final ConnectionEvictionThread underTest = new ConnectionEvictionThread(clientConnectionManager, 1000, 100);
    underTest.start();

    Thread.sleep(300);

    verify(clientConnectionManager, atLeastOnce()).closeExpiredConnections();
    verify(clientConnectionManager, atLeastOnce()).closeIdleConnections(1000, TimeUnit.MILLISECONDS);

    underTest.interrupt();
  }

  /**
   * Verify that eviction thread will continue to run even if calls on ClientConnectionManager fails.
   *
   * @throws Exception unexpected
   */
  @Test
  public void evictionContinuesWhenConnectionManagerFails() throws Exception {
    final HttpClientConnectionManager clientConnectionManager = mock(HttpClientConnectionManager.class);
    doThrow(new RuntimeException("closeExpiredConnections")).when(clientConnectionManager)
        .closeExpiredConnections();
    doThrow(new RuntimeException("closeIdleConnections")).when(clientConnectionManager)
        .closeIdleConnections(1000, TimeUnit.MILLISECONDS);

    final ConnectionEvictionThread underTest = new ConnectionEvictionThread(clientConnectionManager, 1000, 100);
    underTest.start();

    Thread.sleep(300);

    verify(clientConnectionManager, atLeast(2)).closeExpiredConnections();
    verify(clientConnectionManager, atLeast(2)).closeIdleConnections(1000, TimeUnit.MILLISECONDS);

    underTest.interrupt();
  }
}
