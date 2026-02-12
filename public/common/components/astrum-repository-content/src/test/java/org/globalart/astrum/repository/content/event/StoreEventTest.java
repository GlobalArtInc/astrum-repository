
package org.globalart.astrum.repository.content.event;

import org.globalart.astrum.repository.content.AttributeChangeSet;
import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.event.asset.AssetAttributesEvent;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDownloadedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetKindEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPreDeleteEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPrePurgeEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class StoreEventTest
{
  private AssetData asset;

  @Before
  public void setUp() throws Exception {
    ComponentData component = new ComponentData();
    component.setComponentId(1);
    asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setComponent(component);
  }

  @Test
  public void shouldHaveMeaningfulToString() {
    assertEquals(
        "AssetAttributesEvent{changes=[AttributeChange{operation=SET, key='key', value=value} ]} AssetEvent{asset="
            + asset.toString() + "} ContentStoreEvent{contentRepositoryId=1, repository=null}",
        new AssetAttributesEvent(asset, new AttributeChangeSet(AttributeOperation.SET, "key", "value").getChanges())
            .toString());
    assertEquals("AssetCreatedEvent{} AssetEvent{asset=" + asset.toString()
        + "} ContentStoreEvent{contentRepositoryId=1, repository=null}", new AssetCreatedEvent(asset).toString());
    assertEquals("AssetDeletedEvent{} AssetEvent{asset=" + asset.toString()
        + "} ContentStoreEvent{contentRepositoryId=1, repository=null}", new AssetDeletedEvent(asset).toString());
    assertEquals(
        "AssetDownloadedEvent{} AssetEvent{asset=" + asset.toString()
            + "} ContentStoreEvent{contentRepositoryId=1, repository=null}",
        new AssetDownloadedEvent(asset).toString());
    assertEquals("AssetKindEvent{} AssetEvent{asset=" + asset.toString()
        + "} ContentStoreEvent{contentRepositoryId=1, repository=null}", new AssetKindEvent(asset).toString());
    assertEquals("AssetPreDeleteEvent{} AssetEvent{asset=" + asset.toString()
        + "} ContentStoreEvent{contentRepositoryId=1, repository=null}", new AssetPreDeleteEvent(asset).toString());
    assertEquals("AssetPrePurgeEvent{assetIds=[1, 2]} ContentStoreEvent{contentRepositoryId=0, repository=null}",
        new AssetPrePurgeEvent(0, new int[]{1, 2}).toString());
    assertEquals("AssetPurgedEvent{assetIds=[1, 2]} ContentStoreEvent{contentRepositoryId=0, repository=null}",
        new AssetPurgedEvent(0, new int[]{1, 2}).toString());
    assertEquals("AssetUploadedEvent{} AssetEvent{asset=" + asset.toString()
        + "} ContentStoreEvent{contentRepositoryId=1, repository=null}", new AssetUploadedEvent(asset).toString());
  }
}
