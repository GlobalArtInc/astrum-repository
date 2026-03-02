
package org.globalart.astrum.repository.cache;

import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.ExpectedException;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class CacheControllerHolderTest
{
  private static final CacheType TEST = new CacheType("TEST");

  @Rule
  public ExpectedException exception = ExpectedException.none();

  private CacheController contentCacheController = new CacheController(1000, "content");

  private CacheController metadataCacheController = new CacheController(1000, "metadata");

  private CacheControllerHolder underTest;

  @Before
  public void setUp() {
    this.underTest = new CacheControllerHolder(contentCacheController, metadataCacheController);
  }

  @Test
  public void testGetContentCacheController() {
    assertThat(underTest.getContentCacheController(), is(contentCacheController));
  }

  @Test
  public void testGetMetadataCacheController() {
    assertThat(underTest.getMetadataCacheController(), is(metadataCacheController));
  }

  @Test
  public void testGetContentCacheControllerViaGet() {
    assertThat(underTest.get(CacheControllerHolder.CONTENT), is(contentCacheController));
  }

  @Test
  public void testGetMetadataCacheControllerViaGet() {
    assertThat(underTest.get(CacheControllerHolder.METADATA), is(metadataCacheController));
  }

  @Test
  public void testGetUnknownCacheControllerViaGet() {
    assertThat(underTest.get(TEST), is(nullValue()));
  }

  @Test
  public void testGetContentCacheControllerViaRequire() {
    assertThat(underTest.require(CacheControllerHolder.CONTENT), is(contentCacheController));
  }

  @Test
  public void testGetMetadataCacheControllerViaRequire() {
    assertThat(underTest.require(CacheControllerHolder.METADATA), is(metadataCacheController));
  }

  @Test
  public void testGetUnknownCacheControllerViaRequire() {
    exception.expectMessage(TEST.value());
    underTest.require(TEST);
  }
}
