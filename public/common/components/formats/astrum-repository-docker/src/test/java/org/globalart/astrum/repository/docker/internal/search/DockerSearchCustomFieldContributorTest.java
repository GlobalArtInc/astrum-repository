package org.globalart.astrum.repository.docker.internal.search;

import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.Test;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

public class DockerSearchCustomFieldContributorTest
{
  private static final String DIGEST = "sha256:" + "a".repeat(64);

  @Test
  public void indexesWhatTheTaggedManifestCarries() {
    Asset asset = mock(Asset.class);
    when(asset.kind()).thenReturn("TAG");
    when(asset.attributes("docker")).thenReturn(new NestedAttributesMap("docker", Map.of(
        "name", "charts/mychart",
        "digest", DIGEST,
        "mediaType", "application/vnd.oci.image.manifest.v1+json",
        "artifactType", "application/vnd.cncf.helm.config.v1+json",
        "artifactKind", "helm")));
    SearchRecord record = mock(SearchRecord.class);

    new DockerSearchCustomFieldContributor().populateSearchCustomFields(record, asset);

    verify(record).addFormatFieldValue1(DIGEST);
    verify(record).addFormatFieldValue2("application/vnd.oci.image.manifest.v1+json");
    verify(record).addFormatFieldValue3("application/vnd.cncf.helm.config.v1+json");
    verify(record).addFormatFieldValue4("helm");
  }

  @Test
  public void ignoresBlobsAndManifestsStoredUnderADigest() {
    Asset asset = mock(Asset.class);
    when(asset.kind()).thenReturn("BLOB");
    SearchRecord record = mock(SearchRecord.class);

    new DockerSearchCustomFieldContributor().populateSearchCustomFields(record, asset);

    verifyNoInteractions(record);
  }
}
