
package org.globalart.astrum.audit.internal;

import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.ClientInfoProvider;

import org.apache.shiro.subject.Subject;
import org.apache.shiro.util.ThreadContext;
import org.junit.After;
import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link InitiatorProviderImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class InitiatorProviderImplTest
{

  @Mock
  private ClientInfoProvider clientInfoProvider;

  @InjectMocks
  private InitiatorProviderImpl underTest;

  @After
  public void tearDown() {
    reset();
  }

  @Test
  public void testPreferClientInfoWhenAvailable() {
    ClientInfo clientInfo = ClientInfo.builder().userId("foo").remoteIP("1.2.3.4").userAgent("bar").build();
    when(clientInfoProvider.getCurrentThreadClientInfo()).thenReturn(clientInfo);

    String result = underTest.get();
    assertThat(result, equalTo("foo/1.2.3.4"));
  }

  @Test
  public void testClientInfoMissingUsesSubject() {
    ThreadContext.bind(subject("foo"));
    String result = underTest.get();
    assertThat(result, equalTo("foo"));
  }

  private static void reset() {
    ThreadContext.unbindSubject();
    ThreadContext.unbindSecurityManager();
  }

  private static Subject subject(final Object principal) {
    Subject subject = mock(Subject.class);
    when(subject.getPrincipal()).thenReturn(principal);
    return subject;
  }
}
