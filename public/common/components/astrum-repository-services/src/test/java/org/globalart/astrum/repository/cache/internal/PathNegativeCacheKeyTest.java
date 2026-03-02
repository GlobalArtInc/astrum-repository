
package org.globalart.astrum.repository.cache.internal;

import org.globalart.astrum.repository.cache.NegativeCacheKey;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;

public class PathNegativeCacheKeyTest
{
  /**
   * Given: - a parent path cache key - a child key that has a path that starts with parent key path Then: - #isParentOf
   * returns true
   */
  @Test
  public void childKeyShouldStartWithSamePath() {
    PathNegativeCacheKey underTest = new PathNegativeCacheKey("/foo/");
    assertTrue(underTest.isParentOf(new PathNegativeCacheKey("/foo/bar.jar")));
    assertTrue(underTest.isParentOf(new PathNegativeCacheKey("/foo/and/more/bar.jar")));
  }

  /**
   * Given: - a parent path cache key - a child key that has a path that starts with parent key path Then: - #isParentOf
   * returns false
   */
  @Test
  public void pathMustEndWithSlashToBeAParent() {
    PathNegativeCacheKey underTest = new PathNegativeCacheKey("/foo");
    assertFalse(underTest.isParentOf(new PathNegativeCacheKey("/foo/bar.jar")));
    assertFalse(underTest.isParentOf(new PathNegativeCacheKey("/foo/and/more/bar.jar")));
  }

  /**
   * Given: - a parent path cache key - child key is not an {@link PathNegativeCacheKey} Then: - #isParentOf returns
   * false
   */
  @Test
  public void childKeyMustBeOfSameClass() {
    PathNegativeCacheKey underTest = new PathNegativeCacheKey("/foo/bar.jar");
    assertFalse(underTest.isParentOf(mock(NegativeCacheKey.class)));
  }
}
