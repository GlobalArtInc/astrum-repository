
package org.globalart.astrum.internal.web;

import java.io.IOException;
import java.io.UncheckedIOException;

import jakarta.servlet.FilterChain;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.servlet.XFrameOptions;

import org.eclipse.jetty.io.EofException;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static com.google.common.net.HttpHeaders.X_FRAME_OPTIONS;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ErrorPageFilterTest
{
  private ErrorPageFilter underTest;

  @Mock
  private HttpServletRequest request;

  @Mock
  private HttpServletResponse response;

  @Mock
  private FilterChain filterChain;

  @Before
  public void setup() {
    XFrameOptions xFrameOptions = new XFrameOptions(true);
    underTest = new ErrorPageFilter(xFrameOptions);
  }

  @Test
  public void testDoFilter_properXframeOptions() throws Exception {
    doThrow(new IOException("test")).when(filterChain).doFilter(any(), any());

    underTest.doFilter(request, response, filterChain);

    verify(response).setHeader(X_FRAME_OPTIONS, "DENY");
  }

  @Test
  public void testDoFilter_eofExceptionRethrown() throws Exception {
    EofException eofException = new EofException("Client disconnected");
    doThrow(eofException).when(filterChain).doFilter(any(), any());

    assertThrows(EofException.class, () -> underTest.doFilter(request, response, filterChain));
    verifyNoInteractions(response);
  }

  @Test
  public void testDoFilter_wrappedEofExceptionRethrown() throws Exception {
    doThrow(new UncheckedIOException(new EofException("Client disconnected")))
        .when(filterChain)
        .doFilter(any(), any());

    assertThrows(EofException.class, () -> underTest.doFilter(request, response, filterChain));
    verifyNoInteractions(response);
  }

  @Test
  public void testDoFilter_deeplyWrappedEofExceptionRethrown() throws Exception {
    doThrow(new RuntimeException("outer", new UncheckedIOException(new EofException("gone"))))
        .when(filterChain)
        .doFilter(any(), any());

    assertThrows(EofException.class, () -> underTest.doFilter(request, response, filterChain));
    verifyNoInteractions(response);
  }
}
