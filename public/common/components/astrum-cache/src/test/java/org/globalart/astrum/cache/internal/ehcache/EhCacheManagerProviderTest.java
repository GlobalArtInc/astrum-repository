
package org.globalart.astrum.cache.internal.ehcache;

import javax.cache.CacheManager;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.fail;

/**
 * Tests for {@link EhCacheManagerProvider}.
 */
public class EhCacheManagerProviderTest
{
  private EhCacheManagerProvider underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new EhCacheManagerProvider(getClass().getResource("ehcache-test.xml").toURI());
    underTest.start();
  }

  @After
  public void tearDown() throws Exception {
    // safety check that we always cleanup
    if (underTest != null) {
      underTest.stop();
    }
  }

  @Test
  public void basicLifecycle() throws Exception {
    // get should return non-null
    CacheManager cacheManager = underTest.getObject();
    assertThat(cacheManager, notNullValue());

    // repeated get should return same object
    CacheManager cacheManager2 = underTest.getObject();
    assertThat(cacheManager2, is(cacheManager));

    // after stop get should fail
    underTest.stop();
    try {
      underTest.getObject();
      fail();
    }
    catch (IllegalStateException e) {
      // expected
    }
    underTest.start();
  }
}
