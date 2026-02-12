
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Collections;

import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.ExternalMetadata;
import org.globalart.astrum.common.entity.DetachedEntityId;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertEquals;

public class StoreDataTest
{
  private AssetBlobData assetBlob;

  private AssetData asset;

  private ComponentData component;

  private ContentRepositoryData contentRepository;

  @Before
  public void setUp() throws Exception {
    OffsetDateTime time = OffsetDateTime.of(1970, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);

    BlobRef blobRef = new BlobRef("some-node", "some-store", "some-blob");
    assetBlob = new AssetBlobData();
    assetBlob.setAssetBlobId(1);
    assetBlob.setBlobRef(blobRef);
    assetBlob.setBlobSize(1L);
    assetBlob.setContentType("some-contentType");
    assetBlob.setChecksums(Collections.singletonMap("some-algo", "some-checksum"));
    assetBlob.setBlobCreated(time);
    assetBlob.setCreatedBy("some-user");
    assetBlob.setCreatedByIp("some-ip-address");
    assetBlob.setExternalMetadata(new ExternalMetadata("my-etag", time));
    asset = new AssetData();
    asset.setAssetId(1);
    asset.setPath("/some-path");
    asset.setKind("some-kind");
    asset.setAssetBlob(assetBlob);
    asset.setLastDownloaded(time);
    component = new ComponentData();
    component.setComponentId(1);
    component.setNamespace("some-namespace");
    component.setName("some-name");
    component.setKind("some-kind");
    component.setVersion("some-version");
    component.setNormalizedVersion("some-normalized-version");
    asset.setComponent(component);
    contentRepository = new ContentRepositoryData();
    contentRepository.setConfigRepositoryId(new DetachedEntityId("some-id"));
  }

  @Test
  public void shouldHaveMeaningfulToString() {
    String expectedAssetBlob =
        "AssetBlobData{assetBlobId=1, blobRef=some-store@some-blob, blobSize=1, contentType='some-contentType',"
            + " checksums={some-algo=some-checksum}, blobCreated=1970-01-01T00:00Z, createdBy='some-user',"
            + " createdByIp='some-ip-address', externalMetadata='ExternalMetadata[etag=my-etag, lastModified=1970-01-01T00:00Z]'}";
    String expectedComponent =
        "ComponentData{componentId=1, namespace='some-namespace', name='some-name', kind='some-kind', version='some-version', normalizedVersion='some-normalized-version', entityVersion='null'} AbstractRepositoryContent{repositoryId=null, attributes=NestedAttributesMap{parent=null, key='attributes', backing={}}, created=null, lastUpdated=null}";
    String expectedAsset =
        "AssetData{assetId=1, path='/some-path', kind='some-kind', componentId=1, component=%s, assetBlobId=1, assetBlob=%s, lastDownloaded=1970-01-01T00:00Z, assetSize=0} AbstractRepositoryContent{repositoryId=null, attributes=NestedAttributesMap{parent=null, key='attributes', backing={}}, created=null, lastUpdated=null}"
            .formatted(expectedComponent, expectedAssetBlob);

    assertThat(assetBlob.toString(), is(expectedAssetBlob));
    assertThat(asset.toString(), is(expectedAsset));
    assertThat(component.toString(), is(expectedComponent));
    assertEquals(
        "ContentRepositoryData{configRepositoryId=DetachedEntityId{value='some-id'}} AbstractRepositoryContent{repositoryId=null, attributes=NestedAttributesMap{parent=null, key='attributes', backing={}}, created=null, lastUpdated=null}",
        contentRepository.toString());
  }
}
