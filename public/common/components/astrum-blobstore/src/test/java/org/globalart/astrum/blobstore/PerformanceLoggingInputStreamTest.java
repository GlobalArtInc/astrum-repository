
package org.globalart.astrum.blobstore;

import java.io.IOException;
import java.io.InputStream;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class PerformanceLoggingInputStreamTest
{
  @Mock
  private InputStream source;

  @Mock
  private PerformanceLogger logger;

  @InjectMocks
  private PerformanceLoggingInputStream underTest;

  @Test
  public void shouldPassReadsAndCloseToUnderlyingInputStream() throws IOException {
    byte[] buffer = new byte[10];

    when(source.read()).thenReturn(123);
    when(source.read(buffer, 7, 29)).thenReturn(29);

    assertThat(underTest.read(), is(123));
    assertThat(underTest.read(buffer, 7, 29), is(29));
    underTest.close();
    verify(source).close();
  }

  @Test
  public void performanceDataIsLoggedOnClose() throws IOException {
    underTest.close();
    verify(logger).logRead(0, 0);
  }
}
