
package org.globalart.astrum.repository.composer;

import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.CONTENT;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.METADATA;
import static org.globalart.astrum.repository.composer.AssetKind.LIST;
import static org.globalart.astrum.repository.composer.AssetKind.PACKAGES;
import static org.globalart.astrum.repository.composer.AssetKind.PROVIDER;
import static org.globalart.astrum.repository.composer.AssetKind.ZIPBALL;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetKindTest
{
  @Test
  public void cacheTypes() {
    assertThat(LIST.getCacheType(), is(METADATA));
    assertThat(PROVIDER.getCacheType(), is(METADATA));
    assertThat(PACKAGES.getCacheType(), is(METADATA));
    assertThat(ZIPBALL.getCacheType(), is(CONTENT));
  }
}
