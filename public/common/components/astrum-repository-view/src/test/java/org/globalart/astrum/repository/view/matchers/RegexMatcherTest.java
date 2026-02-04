
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link RegexMatcher}.
 */
public class RegexMatcherTest
{
  private Context context(String path) {
    return new Context(mock(Repository.class), new Request.Builder().action("GET").path(path).build());
  }

  @Test
  public void basic() {
    RegexMatcher underTest = new RegexMatcher("foo.*bar");
    assertTrue(underTest.matches(context("foobar")));
    assertTrue(underTest.matches(context("fooooooobar")));
    assertFalse(underTest.matches(context("foobarbaz")));
  }
}
