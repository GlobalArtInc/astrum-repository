
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.Collections;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class FluentAssetQueryImplTest
{
  @Mock
  private ContentFacetSupport facet;

  @Mock
  private AssetStore<?> assetStore;

  @Mock
  private Repository repository;

  private FluentAssetsImpl assets;

  @Before
  public void setUp() {
    Format format = new Format("raw")
    {
    };
    when(repository.getFormat()).thenReturn(format);
    when(repository.getType()).thenReturn(new HostedType());
    when(facet.repository()).thenReturn(repository);
    when(facet.contentRepositoryId()).thenReturn(1);

    assets = new FluentAssetsImpl(facet, assetStore);
  }

  @Test
  public void testCountByKind() {
    when(assetStore.countAssets(eq(1), eq("DOCKER_LAYER"), isNull(), isNull())).thenReturn(12);
    FluentAssetQueryImpl query = new FluentAssetQueryImpl(assets, "DOCKER_LAYER");
    assertThat(query.count(), is(12));
  }

  @Test
  public void testCountByFilter() {
    when(assetStore.countAssets(eq(1), isNull(), eq("path like :path"), any())).thenReturn(7);
    FluentAssetQueryImpl query = new FluentAssetQueryImpl(
        assets, "path like :path", Collections.singletonMap("path", "/test%"));
    assertThat(query.count(), is(7));
  }

  @Test
  public void testBrowseByKind() {
    Continuation continuation = mock(Continuation.class);
    when(assetStore.browseAssets(any(), anyString(), eq("DOCKER_LAYER"), isNull(), isNull(), anyInt()))
        .thenReturn(continuation);

    FluentAssetQueryImpl query = new FluentAssetQueryImpl(assets, "DOCKER_LAYER");
    Continuation<FluentAsset> result = query.browse(10, "token");
    assertThat(result, is(notNullValue()));
  }

  @Test(expected = UnsupportedOperationException.class)
  public void testBrowseEagerThrowsUnsupported() {
    FluentAssetQueryImpl query = new FluentAssetQueryImpl(assets, "kind");
    query.browseEager(10, null);
  }
}
