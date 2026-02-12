
package org.globalart.astrum.repository.content.event.asset;

import org.globalart.astrum.repository.content.store.AssetData;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class AssetPathChangedEventTest
{
  @Test
  public void testEventConstruction() {
    AssetData asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setAssetId(100);
    asset.setPath("/new/path/artifact.jar");

    String oldPath = "/old/path/artifact.jar";
    String newPath = "/new/path/artifact.jar";

    AssetPathChangedEvent event = new AssetPathChangedEvent(asset, oldPath, newPath);

    assertEquals(asset, event.getAsset());
    assertEquals(oldPath, event.getOldPath());
    assertEquals(newPath, event.getNewPath());
  }

  @Test
  public void testEventExtendsAssetUpdatedEvent() {
    AssetData asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setAssetId(100);
    asset.setPath("/new/path");

    AssetPathChangedEvent event = new AssetPathChangedEvent(asset, "/old", "/new");

    assertTrue(event instanceof AssetUpdatedEvent);
  }

  @Test
  public void testToString() {
    AssetData asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setAssetId(100);
    asset.setPath("/new/path");

    AssetPathChangedEvent event = new AssetPathChangedEvent(asset, "/old", "/new");

    String str = event.toString();
    assertNotNull(str);
    assertTrue(str.contains("AssetPathChangedEvent"));
    assertTrue(str.contains("/old"));
    assertTrue(str.contains("/new"));
  }

  @Test
  public void testEventWithNullPaths() {
    AssetData asset = new AssetData();
    asset.setRepositoryId(1);
    asset.setAssetId(100);
    asset.setPath("/path");

    AssetPathChangedEvent event = new AssetPathChangedEvent(asset, null, "/path");

    assertEquals(null, event.getOldPath());
    assertEquals("/path", event.getNewPath());
  }
}
