
package org.globalart.astrum.security;

import org.apache.shiro.subject.Subject;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.UserIdHelper.UNKNOWN;

/**
 * Tests for {@link UserIdHelper}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class UserIdHelperTest
{
  private Subject subject(final Object principal) {
    Subject subject = mock(Subject.class);
    when(subject.getPrincipal()).thenReturn(principal);
    return subject;
  }

  @Test
  public void get_subject() {
    assertThat(UserIdHelper.get(subject("test")), is("test"));
  }

  @Test
  public void get_nullSubject() {
    assertThat(UserIdHelper.get(null), is(UNKNOWN));
  }

  @Test
  public void get_nullPrincipal() {
    assertThat(UserIdHelper.get(subject(null)), is(UNKNOWN));
  }
}
