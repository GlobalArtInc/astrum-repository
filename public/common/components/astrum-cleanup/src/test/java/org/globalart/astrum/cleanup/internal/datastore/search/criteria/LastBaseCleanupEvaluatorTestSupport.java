
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
public abstract class LastBaseCleanupEvaluatorTestSupport
{
  @Mock
  protected Repository repository;

  @BeforeEach
  void setup() {
    setupMocks();
  }

  protected abstract void setupMocks();

  protected Asset createMockAssetWithBlob(final OffsetDateTime blobCreatedTime) {
    Asset asset = mock(Asset.class);
    AssetBlob assetBlob = mock(AssetBlob.class);
    when(asset.blob()).thenReturn(Optional.of(assetBlob));
    when(assetBlob.blobCreated()).thenReturn(blobCreatedTime);
    return asset;
  }

  protected Asset createMockAssetWithLastDownloaded(final OffsetDateTime lastDownloadedTime) {
    Asset asset = mock(Asset.class);
    when(asset.lastDownloaded()).thenReturn(Optional.of(lastDownloadedTime));
    return asset;
  }

  protected Component createMockComponent() {
    return mock(Component.class);
  }
}
