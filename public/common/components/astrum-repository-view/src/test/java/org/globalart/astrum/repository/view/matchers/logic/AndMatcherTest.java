
package org.globalart.astrum.repository.view.matchers.logic;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.matchers.RegexMatcher;

import java.util.Arrays;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link AndMatcher}.
 */
public class AndMatcherTest
{
  private Context context(String path) {
    return new Context(mock(Repository.class), new Request.Builder().action("GET").path(path).build());
  }

  @Test
  public void andTwoMatchers() {
    AndMatcher underTest = new AndMatcher(Arrays.asList(
        new RegexMatcher("foo.*"),
        new RegexMatcher(".*bar")));

    assertTrue(underTest.matches(context("foobar")));
    assertTrue(underTest.matches(context("fooooooobar")));
    assertFalse(underTest.matches(context("foobarbaz")));
  }
}
