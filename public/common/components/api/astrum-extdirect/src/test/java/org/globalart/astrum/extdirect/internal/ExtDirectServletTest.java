
package org.globalart.astrum.extdirect.internal;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link ExtDirectServlet}.
 */
@RunWith(MockitoJUnitRunner.class)
public class ExtDirectServletTest
{
  @Mock
  private ApplicationDirectories directories;

  @Mock
  private ExtDirectDispatcher extDirectDispatcher;

  @Mock
  private HttpServletRequest request;

  @Mock
  private HttpServletResponse response;

  private ExtDirectServlet servlet;

  @Before
  public void setUp() {
    servlet = new ExtDirectServlet(directories, extDirectDispatcher, false);
  }

  @Test
  public void wrapRequest_normalizesNullPathInfoToEmpty() {
    when(request.getPathInfo()).thenReturn(null);
    assertThat(servlet.wrapRequest(request).getPathInfo(), is(""));
  }

  @Test
  public void wrapRequest_preservesNonNullPathInfo() {
    when(request.getPathInfo()).thenReturn("/some/path");
    assertThat(servlet.wrapRequest(request).getPathInfo(), is("/some/path"));
  }

  @Test
  public void doPost_setsXFrameOptionsHeader_whenPathInfoIsNull() throws Exception {
    when(request.getPathInfo()).thenReturn(null);
    when(request.getContentType()).thenReturn("application/json");

    assertThrows(IllegalStateException.class, () -> servlet.doPost(request, response));

    verify(response).setHeader("X-Frame-Options", "DENY");
  }
}
