
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link RepositoryDoesNotExistException}.
 */
public class RepositoryDoesNotExistExceptionTest
{
  @Test
  public void testConstructorAndMessage() {
    RepositoryDoesNotExistException exception = new RepositoryDoesNotExistException();

    assertThat(exception.getMessage(), is("Repository does not exist"));
    assertThat(exception.getCause(), is(nullValue()));
  }

  @Test
  public void testIsCheckedException() {
    RepositoryDoesNotExistException exception = new RepositoryDoesNotExistException();

    assertThat(exception instanceof Exception, is(true));
    assertThat(RuntimeException.class.isInstance(exception), is(false));
  }
}
