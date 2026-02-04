
package org.globalart.astrum.repository.view.matchers;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link SuffixMatcher}.
 */
public class SuffixMatcherTest
{
  private Context context(String path) {
    return new Context(mock(Repository.class), new Request.Builder().action("GET").path(path).build());
  }

  @Test
  public void basic() {
    SuffixMatcher underTest = new SuffixMatcher("/");
    assertTrue(underTest.matches(context("foo/")));
    assertFalse(underTest.matches(context("foo/index.html")));
  }
}
