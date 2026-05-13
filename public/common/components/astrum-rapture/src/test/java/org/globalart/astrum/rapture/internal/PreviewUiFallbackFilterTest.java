
package org.globalart.astrum.rapture.internal;

import jakarta.servlet.FilterChain;
import jakarta.servlet.RequestDispatcher;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class PreviewUiFallbackFilterTest
{
  @Mock
  private HttpServletRequest httpRequest;

  @Mock
  private HttpServletResponse httpResponse;

  @Mock
  private FilterChain filterChain;

  @Mock
  private RequestDispatcher requestDispatcher;

  @Mock
  private ServletRequest genericRequest;

  private PreviewUiFallbackFilter filter;

  @Before
  public void setUp() {
    filter = new PreviewUiFallbackFilter();
  }

  @Test
  public void testDoFilter_forwardsPreviewRequestToIndexHtml() throws Exception {
    when(httpRequest.getRequestURI()).thenReturn("/preview/browse/welcome");
    when(httpRequest.getRequestDispatcher("/")).thenReturn(requestDispatcher);

    filter.doFilter(httpRequest, httpResponse, filterChain);

    verify(httpRequest).getRequestDispatcher("/");
    verify(requestDispatcher).forward(httpRequest, httpResponse);
    verify(filterChain, never()).doFilter(httpRequest, httpResponse);
  }

  @Test
  public void testDoFilter_forwardsNestedPreviewRequestToIndexHtml() throws Exception {
    when(httpRequest.getRequestURI())
        .thenReturn("/preview/browse/search/component/maven:org.apache:commons-lang3/overview");
    when(httpRequest.getRequestDispatcher("/")).thenReturn(requestDispatcher);

    filter.doFilter(httpRequest, httpResponse, filterChain);

    verify(httpRequest).getRequestDispatcher("/");
    verify(requestDispatcher).forward(httpRequest, httpResponse);
    verify(filterChain, never()).doFilter(httpRequest, httpResponse);
  }

  @Test
  public void testDoFilter_forwardsPreviewAdminRequestToIndexHtml() throws Exception {
    when(httpRequest.getRequestURI()).thenReturn("/preview/admin/repository/repositories");
    when(httpRequest.getRequestDispatcher("/")).thenReturn(requestDispatcher);

    filter.doFilter(httpRequest, httpResponse, filterChain);

    verify(httpRequest).getRequestDispatcher("/");
    verify(requestDispatcher).forward(httpRequest, httpResponse);
    verify(filterChain, never()).doFilter(httpRequest, httpResponse);
  }

  @Test
  public void testDoFilter_nonHttpRequestPassesThrough() throws Exception {
    filter.doFilter(genericRequest, httpResponse, filterChain);

    verify(filterChain).doFilter(genericRequest, httpResponse);
  }
}
