
package org.globalart.astrum.security.anonymous;

import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.subject.FakeAlmightySubject;

import org.apache.shiro.util.ThreadContext;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AnonymousFilterTest
{
  private AnonymousFilter underTest;

  private static String ANONYMOUS_USER = "anonymousUser";

  @Mock
  private Provider<AnonymousManager> anonymousManagerProvider;

  @Mock
  private Provider<EventManager> eventManager;

  @Mock
  private HttpServletRequest request;

  @Mock
  private AnonymousManager anonymousManager;

  @Before
  public void setup() {
    underTest = new AnonymousFilter(anonymousManagerProvider, eventManager);
    ThreadContext.bind(FakeAlmightySubject.forUserId(ANONYMOUS_USER));
  }

  @Test
  public void testBuildSubjectWhenIsAnonymousUser() throws Exception {
    when(eventManager.get()).thenReturn(mock(EventManager.class));
    when(anonymousManagerProvider.get()).thenReturn(anonymousManager);
    when(anonymousManager.isEnabled()).thenReturn(true);
    when(anonymousManager.getConfiguration()).thenReturn(mock(AnonymousConfiguration.class));
    when(anonymousManager.getConfiguration().getUserId()).thenReturn(ANONYMOUS_USER);

    underTest.preHandle(request, null);

    verify(anonymousManager).buildSubject();
  }

}
