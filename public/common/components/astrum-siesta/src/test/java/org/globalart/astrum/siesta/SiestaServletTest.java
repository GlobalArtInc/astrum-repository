
package org.globalart.astrum.siesta;

import java.io.IOException;

import jakarta.servlet.ServletConfig;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SiestaServletTest
{
  private SiestaServlet underTest;

  @Mock
  private ComponentContainer componentContainer;

  @Mock
  private ServletConfig servletConfig;

  @Mock
  private HttpServletRequest request;

  @Mock
  private HttpServletResponse response;

  @BeforeEach
  void setup() {
    underTest = new SiestaServlet(componentContainer);
  }

  @Test
  void testInit() throws ServletException {
    underTest.init(servletConfig);
    verify(componentContainer).init(servletConfig);
  }

  @Test
  void testService() throws ServletException, IOException {
    when(request.getRequestURI()).thenReturn("/test");

    underTest.service(request, response);

    verify(componentContainer).service(request, response);
  }

  @Test
  void testDestroy() {
    underTest.destroy();
    verify(componentContainer).destroy();
  }
}
