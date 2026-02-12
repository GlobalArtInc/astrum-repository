
package org.globalart.astrum.repository.content.browse.store;

import java.util.Optional;

import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.blobstore.DefaultBlobIdLocationResolver;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.event.asset.AssetAttributesEvent;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.internal.AssetAuditor;

import org.junit.Test;
import org.mockito.MockedStatic;
import org.mockito.Mockito;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class AssetAuditorTest
{
  AssetAuditor assetAuditor;

  @Test
  public void testAuditChangesDetailEnabled() {
    assetAuditor = Mockito.spy(new AssetAuditor(true, new DefaultBlobIdLocationResolver()));

    AssetData assetData = new AssetData();
    assetData.setRepositoryId(1);

    Repository repository = mock(Repository.class);

    AssetAttributesEvent assetAttributesEvent = mock(AssetAttributesEvent.class);
    when(assetAttributesEvent.getAsset()).thenReturn(assetData);
    when(assetAttributesEvent.getRepository()).thenReturn(Optional.of(repository));

    AuditRecorder auditRecorder = mock(AuditRecorder.class);
    when(auditRecorder.isEnabled()).thenReturn(true);
    assetAuditor.setAuditRecorder(auditRecorder);

    try (MockedStatic<EventHelper> mockedStatic = mockStatic(EventHelper.class)) {
      mockedStatic.when(EventHelper::isReplicating).thenReturn(false);
      assetAuditor.on(assetAttributesEvent);
      verify(assetAttributesEvent, times(1)).getChanges();
    }

    verify(auditRecorder, times(1)).record(any());
  }

  @Test
  public void testAuditChangesDetailDisabled() {
    assetAuditor = Mockito.spy(new AssetAuditor(false, new DefaultBlobIdLocationResolver()));

    AssetData assetData = new AssetData();
    assetData.setRepositoryId(1);

    Repository repository = mock(Repository.class);

    AssetAttributesEvent assetAttributesEvent = mock(AssetAttributesEvent.class);
    when(assetAttributesEvent.getAsset()).thenReturn(assetData);
    when(assetAttributesEvent.getRepository()).thenReturn(Optional.of(repository));

    AuditRecorder auditRecorder = mock(AuditRecorder.class);
    when(auditRecorder.isEnabled()).thenReturn(true);

    assetAuditor.setAuditRecorder(auditRecorder);

    try (MockedStatic<EventHelper> mockedStatic = mockStatic(EventHelper.class)) {
      mockedStatic.when(EventHelper::isReplicating).thenReturn(false);
      assetAuditor.on(assetAttributesEvent);
      verify(assetAttributesEvent, times(0)).getChanges();
    }

    verify(auditRecorder, times(1)).record(any());

  }
}
