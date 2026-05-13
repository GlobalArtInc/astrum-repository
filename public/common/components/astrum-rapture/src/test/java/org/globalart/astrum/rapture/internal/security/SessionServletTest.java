
package org.globalart.astrum.rapture.internal.security;

import java.util.Arrays;
import java.util.HashSet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.event.EventManager;

import org.apache.shiro.session.Session;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.Subject;
import org.apache.shiro.util.ThreadContext;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static com.google.common.net.HttpHeaders.X_FRAME_OPTIONS;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SessionServletTest
{
  @Mock
  private HttpServletRequest httpServletRequest;

  @Mock
  private HttpServletResponse httpServletResponse;

  @Mock
  private Subject subject;

  @Mock
  private EventManager eventManager;

  private SessionServlet underTest;

  @Before
  public void setup() {
    underTest = new SessionServlet(eventManager);

    PrincipalCollection principalCollection = mock(PrincipalCollection.class);
    when(subject.getPrincipals()).thenReturn(principalCollection);
    when(principalCollection.getRealmNames()).thenReturn(new HashSet<>(Arrays.asList("realm")));
    when(subject.getPrincipal()).thenReturn("someuser");

    when(subject.isAuthenticated()).thenReturn(true);
    when(subject.getSession(false)).thenReturn(mock(Session.class));
    ThreadContext.bind(subject);
  }

  @After
  public void cleanup() {
    ThreadContext.unbindSubject();
  }

  @Test
  public void testDoPost() throws Exception {
    underTest.doPost(httpServletRequest, httpServletResponse);

    verify(httpServletResponse).setHeader(X_FRAME_OPTIONS, "DENY");
  }

  @Test
  public void testDoDelete() throws Exception {
    when(subject.isAuthenticated()).thenReturn(false);
    when(subject.isRemembered()).thenReturn(false);
    when(subject.getSession(false)).thenReturn(null);

    underTest.doDelete(httpServletRequest, httpServletResponse);

    verify(httpServletResponse).setHeader(X_FRAME_OPTIONS, "DENY");
  }

  /**
   * ASTRUM-50529: Test that doDelete handles expired/anonymous sessions gracefully
   * without throwing NullPointerException when getPrincipal() returns null.
   */
  @Test
  public void testDoDeleteWithExpiredSessionHandledGracefully() throws Exception {
    // Simulate an expired session - principal is null
    when(subject.getPrincipal()).thenReturn(null);

    underTest.doDelete(httpServletRequest, httpServletResponse);

    // Verify response is still successful
    verify(httpServletResponse).setStatus(204); // SC_NO_CONTENT
    verify(httpServletResponse).setHeader(X_FRAME_OPTIONS, "DENY");

    // Verify that logout was not called (user already logged out)
    verify(subject, never()).logout();
    verify(eventManager, never()).post(any());
  }

  /**
   * ASTRUM-50529: Test that doDelete handles the case where subject has null principals
   * (another way session timeout can manifest).
   */
  @Test
  public void testDoDeleteWithNullPrincipalsHandledGracefully() throws Exception {
    // Simulate null principals
    when(subject.getPrincipal()).thenReturn(null);
    when(subject.getPrincipals()).thenReturn(null);

    underTest.doDelete(httpServletRequest, httpServletResponse);

    // Should complete without NPE
    verify(httpServletResponse).setStatus(204);
    verify(httpServletResponse).setHeader(X_FRAME_OPTIONS, "DENY");
  }
}
