
package org.globalart.astrum.repository.content.rest;

import java.time.OffsetDateTime;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityMetadata;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetBlobData;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertNotNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetXOBuilderTest
{
  @Mock
  Repository repository;

  @Mock
  EntityMetadata assetOneEntityMetadata;

  @Mock
  EntityId assetOneEntityId;

  private static final String ASSET_PATH = "/nameOne";

  private static final int AN_ASSET_ID = 1;

  @Before
  public void setup() {
    when(assetOneEntityMetadata.getId()).thenReturn(assetOneEntityId);
    when(assetOneEntityId.getValue()).thenReturn("assetOne");

    when(repository.getName()).thenReturn("maven-releases");
    when(repository.getUrl()).thenReturn("http://localhost:8081/repository/maven-releases");
    when(repository.getFormat()).thenReturn(new Format("maven2")
    {
    });
  }

  @Test
  public void blobCreatedExists() {
    AssetXO assetXO = AssetXOBuilder.fromAsset(anAsset(), repository, null, false);

    assertThat(assetXO.getId(), notNullValue());
    assertThat(assetXO.getPath(), is("/nameOne"));
    assertThat(assetXO.getDownloadUrl(), is("http://localhost:8081/repository/maven-releases/nameOne"));
    assertNotNull(assetXO.getBlobCreated());
  }

  @Test
  public void fromAsset_useAssetCreatedForBlobCreated() {
    // For backward compatibility, fromAsset should use asset.created() for blobCreated
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);
    OffsetDateTime blobCreated = OffsetDateTime.now().minusHours(1);

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated);
    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    // Should use asset.created() timestamp
    assertThat(assetXO.getBlobCreated().toInstant().toEpochMilli(),
        is(assetCreated.toInstant().toEpochMilli()));
  }

  @Test
  public void fromEagerAsset_useActualBlobCreatedTimestamp() {
    // fromEagerAsset should use the actual blob_created timestamp from AssetBlob
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);
    OffsetDateTime blobCreated = OffsetDateTime.now().minusHours(1);

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated);
    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    // Should use blob.blobCreated() timestamp, not asset.created()
    assertThat(assetXO.getBlobCreated().toInstant().toEpochMilli(),
        is(blobCreated.toInstant().toEpochMilli()));
  }

  @Test
  public void fromEagerAsset_fallbackToAssetCreatedWhenBlobMissing() {
    // If blob is not present, should fall back to asset.created()
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);

    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(assetCreated);
    // No blob set

    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    // Should fall back to asset.created() when blob is missing
    assertThat(assetXO.getBlobCreated().toInstant().toEpochMilli(),
        is(assetCreated.toInstant().toEpochMilli()));
  }

  @Test
  public void fromEagerAsset_allFieldsPopulated() {
    // Verify fromEagerAsset populates all standard fields correctly
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);
    OffsetDateTime blobCreated = OffsetDateTime.now().minusHours(1);

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated);
    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    assertThat(assetXO.getId(), notNullValue());
    assertThat(assetXO.getPath(), is(ASSET_PATH));
    assertThat(assetXO.getDownloadUrl(), is("http://localhost:8081/repository/maven-releases" + ASSET_PATH));
    assertThat(assetXO.getRepository(), is("maven-releases"));
    assertThat(assetXO.getFormat(), is("maven2"));
    assertNotNull(assetXO.getBlobCreated());
  }

  @Test
  public void fromAsset_blobStoreNamePopulated() {
    // Verify that blobStoreName is populated in AssetXO from BlobRef
    OffsetDateTime assetCreated = OffsetDateTime.now();
    OffsetDateTime blobCreated = OffsetDateTime.now();

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated,
        "default@051ae249-9d2d-4807-85d0-9c920198b3b7@2025-11-13T09:31");

    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobStoreName(), is("default"));
  }

  @Test
  public void fromEagerAsset_blobStoreNamePopulated() {
    // Verify that blobStoreName is populated in AssetXO from BlobRef for eager loading
    OffsetDateTime assetCreated = OffsetDateTime.now();
    OffsetDateTime blobCreated = OffsetDateTime.now();

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated,
        "test-blob@14c05db1-4329-4733-a5de-2ee6fa5c46c2@2025-11-17T07:55");

    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobStoreName(), is("test-blob"));
  }

  @Test
  public void fromAsset_blobUpdatedPopulated() {
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);
    OffsetDateTime blobCreated = OffsetDateTime.now().minusHours(1);

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated);
    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobUpdated(), notNullValue());
    assertThat(assetXO.getBlobUpdated().toInstant().toEpochMilli(),
        is(blobCreated.toInstant().toEpochMilli()));
  }

  @Test
  public void fromEagerAsset_blobUpdatedPopulated() {
    OffsetDateTime assetCreated = OffsetDateTime.now().minusHours(2);
    OffsetDateTime blobCreated = OffsetDateTime.now().minusHours(1);

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated);
    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobUpdated(), notNullValue());
    assertThat(assetXO.getBlobUpdated().toInstant().toEpochMilli(),
        is(blobCreated.toInstant().toEpochMilli()));
  }

  @Test
  public void fromAsset_blobRefPopulated() {
    OffsetDateTime assetCreated = OffsetDateTime.now();
    OffsetDateTime blobCreated = OffsetDateTime.now();
    String blobRefString = "test-blob@14c05db1-4329-4733-a5de-2ee6fa5c46c2@2025-11-17T07:55";

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated, blobRefString);
    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobRef(), is(blobRefString));
  }

  @Test
  public void fromEagerAsset_blobRefPopulated() {
    OffsetDateTime assetCreated = OffsetDateTime.now();
    OffsetDateTime blobCreated = OffsetDateTime.now();
    String blobRefString = "test-blob@14c05db1-4329-4733-a5de-2ee6fa5c46c2@2025-11-17T07:55";

    Asset asset = anAssetWithTimestamps(assetCreated, blobCreated, blobRefString);
    AssetXO assetXO = AssetXOBuilder.fromEagerAsset(asset, repository, null, false);

    assertThat(assetXO.getBlobRef(), is(blobRefString));
  }

  @Test
  public void fromAsset_lastVerifiedExtractedFromCache() {
    OffsetDateTime assetCreated = OffsetDateTime.now();
    OffsetDateTime blobCreated = OffsetDateTime.now();
    long lastVerifiedTimestamp = System.currentTimeMillis();

    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(assetCreated);

    AssetBlobData assetBlob = new AssetBlobData();
    assetBlob.setAssetBlobId(1);
    assetBlob.setBlobCreated(blobCreated);
    asset.setAssetBlob(assetBlob);

    // Add cache attributes with last_verified
    asset.attributes("cache").set("last_verified", lastVerifiedTimestamp);

    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    assertThat(assetXO.getLastVerified(), notNullValue());
    assertThat(assetXO.getLastVerified().getTime(), is(lastVerifiedTimestamp));
  }

  @Test
  public void fromAsset_lastVerifiedNullWhenNotPresent() {
    Asset asset = anAsset();
    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    // Should not throw exception when cache.last_verified is not present
    // This is normal for hosted repos
    assertThat(assetXO.getLastVerified(), is(nullValue()));
  }

  @Test
  public void fromAsset_uploaderFieldsPresent_whenVisible() {
    OffsetDateTime now = OffsetDateTime.now();
    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(now);

    AssetBlobData blob = new AssetBlobData();
    blob.setAssetBlobId(1);
    blob.setBlobCreated(now);
    blob.setCreatedBy("testuser");
    blob.setCreatedByIp("192.168.1.1");
    asset.setAssetBlob(blob);

    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, true);

    assertThat(assetXO.getUploader(), is("testuser"));
    assertThat(assetXO.getUploaderIp(), is("192.168.1.1"));
  }

  @Test
  public void fromAsset_uploaderFieldsNull_whenNotVisible() {
    OffsetDateTime now = OffsetDateTime.now();
    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(now);

    AssetBlobData blob = new AssetBlobData();
    blob.setAssetBlobId(1);
    blob.setBlobCreated(now);
    blob.setCreatedBy("testuser");
    blob.setCreatedByIp("192.168.1.1");
    asset.setAssetBlob(blob);

    AssetXO assetXO = AssetXOBuilder.fromAsset(asset, repository, null, false);

    assertThat(assetXO.getUploader(), is(nullValue()));
    assertThat(assetXO.getUploaderIp(), is(nullValue()));
  }

  @Test
  public void fromAsset_registryUrl_populatedFromDescriptor() {
    AssetXODescriptor descriptor = mock(AssetXODescriptor.class);
    when(descriptor.computeRegistryUrl(repository)).thenReturn("registry.example.com:5000");
    when(descriptor.listExposedAttributeKeys()).thenReturn(Set.of());

    Map<String, AssetXODescriptor> descriptors = Map.of("maven2", descriptor);
    AssetXO assetXO = AssetXOBuilder.fromAsset(anAsset(), repository, descriptors, false);

    assertThat(assetXO.getRegistryUrl(), is("registry.example.com:5000"));
  }

  @Test
  public void fromAsset_registryUrl_nullWhenDescriptorsNull() {
    AssetXO assetXO = AssetXOBuilder.fromAsset(anAsset(), repository, null, false);

    assertThat(assetXO.getRegistryUrl(), is(nullValue()));
  }

  @Test
  public void fromAsset_registryUrl_nullWhenDescriptorReturnsNull() {
    AssetXODescriptor descriptor = mock(AssetXODescriptor.class);
    when(descriptor.computeRegistryUrl(repository)).thenReturn(null);
    when(descriptor.listExposedAttributeKeys()).thenReturn(Set.of());

    Map<String, AssetXODescriptor> descriptors = Map.of("maven2", descriptor);
    AssetXO assetXO = AssetXOBuilder.fromAsset(anAsset(), repository, descriptors, false);

    assertThat(assetXO.getRegistryUrl(), is(nullValue()));
  }

  private Asset anAsset() {
    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(OffsetDateTime.now());
    AssetBlobData assetBlob = new AssetBlobData();
    assetBlob.setAssetBlobId(1);
    asset.setAssetBlob(assetBlob);
    return asset;
  }

  private Asset anAssetWithTimestamps(OffsetDateTime assetCreated, OffsetDateTime blobCreated) {
    return anAssetWithTimestamps(assetCreated, blobCreated, null);
  }

  private Asset anAssetWithTimestamps(OffsetDateTime assetCreated, OffsetDateTime blobCreated, String blobRefString) {
    AssetData asset = new AssetData();
    asset.setAssetId(AN_ASSET_ID);
    asset.setPath(ASSET_PATH);
    asset.setCreated(assetCreated);

    AssetBlobData assetBlob = new AssetBlobData();
    assetBlob.setAssetBlobId(1);
    assetBlob.setBlobCreated(blobCreated);
    if (blobRefString != null) {
      assetBlob.setBlobRef(org.globalart.astrum.blobstore.api.BlobRef.parse(blobRefString));
    }
    asset.setAssetBlob(assetBlob);

    return asset;
  }
}
