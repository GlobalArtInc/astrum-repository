
package org.globalart.astrum.repository.httpclient.internal;

import java.io.IOException;

import org.globalart.astrum.repository.httpclient.AutoBlockConfiguration;
import org.globalart.astrum.repository.httpclient.HttpClientConfig;

import org.apache.http.HttpHost;
import org.apache.http.conn.ConnectionPoolTimeoutException;
import org.apache.http.impl.client.CloseableHttpClient;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.AUTO_BLOCKED_UNAVAILABLE;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.READY;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.UNAVAILABLE;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlockingHttpClientRecordFailureTest
{
  @Mock
  CloseableHttpClient httpClient;

  @Mock
  AutoBlockConfiguration autoBlockConfiguration;

  BlockingHttpClient underTest;

  HttpHost target;

  @Before
  public void setUp() {
    target = new HttpHost("blackhole-1.iana.org", 443, "https");
  }

  private BlockingHttpClient newClient(final boolean autoBlock) {
    HttpClientConfig config = new HttpClientConfig();
    config.autoBlock = autoBlock;
    return new BlockingHttpClient(httpClient, config,
        (oldStatus, newStatus) -> {
        }, true, autoBlockConfiguration);
  }

  @After
  public void tearDown() throws Exception {
    if (underTest != null) {
      underTest.close();
    }
  }

  @Test
  public void recordConnectionFailure_setsBlockedUntil_andAutoBlockedStatus() {
    underTest = newClient(true);

    underTest.recordConnectionFailure(new IOException("connect refused"), target);

    assertThat(underTest.getStatus().getType(), equalTo(AUTO_BLOCKED_UNAVAILABLE));
    assertThat(underTest.getStatus().getBlockedUntil() != null, equalTo(true));
  }

  @Test
  public void recordConnectionFailure_ignoresConnectionPoolTimeout() {
    underTest = newClient(true);

    underTest.recordConnectionFailure(
        new ConnectionPoolTimeoutException("pool exhausted"), target);

    assertThat(underTest.getStatus().getType(), equalTo(READY));
    assertThat(underTest.getStatus().getBlockedUntil() == null, equalTo(true));
  }

  @Test
  public void recordConnectionFailure_autoBlockDisabled_setsUnavailableStatus() {
    underTest = newClient(false);

    underTest.recordConnectionFailure(new IOException("connect refused"), target);

    assertThat(underTest.getStatus().getType(), equalTo(UNAVAILABLE));
    assertThat(underTest.getStatus().getBlockedUntil() == null, equalTo(true));
  }
}
