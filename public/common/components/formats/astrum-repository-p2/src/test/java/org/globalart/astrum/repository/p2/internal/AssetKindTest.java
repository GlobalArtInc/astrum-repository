
package org.globalart.astrum.repository.p2.internal;

import org.junit.Test;

import static org.globalart.astrum.repository.cache.CacheControllerHolder.CONTENT;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.METADATA;
import static org.globalart.astrum.repository.p2.internal.AssetKind.ARTIFACTS_METADATA;
import static org.globalart.astrum.repository.p2.internal.AssetKind.BINARY_BUNDLE;
import static org.globalart.astrum.repository.p2.internal.AssetKind.BUNDLE;
import static org.globalart.astrum.repository.p2.internal.AssetKind.COMPOSITE_ARTIFACTS;
import static org.globalart.astrum.repository.p2.internal.AssetKind.COMPOSITE_CONTENT;
import static org.globalart.astrum.repository.p2.internal.AssetKind.P2_INDEX;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsEqual.equalTo;

public class AssetKindTest
{
  @Test
  public void cacheTypes() throws Exception {
    assertThat(P2_INDEX.getCacheType(), is(equalTo(METADATA)));
    assertThat(AssetKind.CONTENT_METADATA.getCacheType(), is(equalTo(METADATA)));
    assertThat(ARTIFACTS_METADATA.getCacheType(), is(equalTo(METADATA)));
    assertThat(COMPOSITE_ARTIFACTS.getCacheType(), is(equalTo(METADATA)));
    assertThat(COMPOSITE_CONTENT.getCacheType(), is(equalTo(METADATA)));
    assertThat(BUNDLE.getCacheType(), is(equalTo(CONTENT)));
    assertThat(BINARY_BUNDLE.getCacheType(), is(equalTo(CONTENT)));
  }
}
