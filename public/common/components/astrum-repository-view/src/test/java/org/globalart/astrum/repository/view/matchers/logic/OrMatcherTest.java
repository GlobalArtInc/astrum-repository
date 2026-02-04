
package org.globalart.astrum.repository.view.matchers.logic;

import java.util.Arrays;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link OrMatcher}.
 */
public class OrMatcherTest
{
  private Context context(String path) {
    return new Context(mock(Repository.class), new Request.Builder().action("GET").path(path).build());
  }

  @Test
  public void orTwoMatchers() {
    OrMatcher underTest = new OrMatcher(Arrays.asList(
        new LiteralMatcher("foo"),
        new LiteralMatcher("bar")));

    assertTrue(underTest.matches(context("foo")));
    assertTrue(underTest.matches(context("bar")));
    assertFalse(underTest.matches(context("baz")));
  }
}
