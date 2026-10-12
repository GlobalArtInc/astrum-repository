package org.globalart.astrum.repository.yum.internal.proxy;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.internal.repodata.Repomd;
import org.globalart.astrum.repository.yum.internal.repodata.RepomdData;

import org.junit.Before;
import org.junit.Test;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class YumProxyFacetTest
{
  private final YumContentFacet content = mock(YumContentFacet.class);

  private final YumProxyFacet underTest = new YumProxyFacet();

  @Before
  public void setUp() throws Exception {
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn("yum-proxy");
    when(repository.facet(YumContentFacet.class)).thenReturn(content);
    underTest.attach(repository);
  }

  @Test
  public void marksCachedMetadataStaleWhenTheNewRepomdListsAnotherChecksum() throws Exception {
    FluentAsset changed = cached("/el9/repodata/primary.xml.gz", "sha256", "old");
    FluentAsset unchanged = cached("/el9/repodata/filelists.xml.gz", "sha256", "same");
    FluentAsset sha1 = cached("/el9/repodata/other.xml.gz", "sha1", "abc");
    when(content.getAsset("/el9/repodata/updateinfo.xml.gz")).thenReturn(Optional.empty());

    underTest.invalidateChangedMetadata("/el9/repodata/repomd.xml", repomd(List.of(
        data("primary", "sha256", "new", "repodata/primary.xml.gz"),
        data("filelists", "sha256", "same", "repodata/filelists.xml.gz"),
        data("other", "sha", "ABC", "repodata/other.xml.gz"),
        data("updateinfo", "sha256", "x", "repodata/updateinfo.xml.gz"))));

    verify(changed).markAsStale();
    verify(unchanged, never()).markAsStale();
    verify(sha1, never()).markAsStale();
  }

  @Test
  public void ignoresRepomdThatCannotBeRead() {
    underTest.invalidateChangedMetadata("/repodata/repomd.xml",
        new Content(new BytesPayload("not xml".getBytes(), "application/xml")));

    verify(content, never()).getAsset("/repodata/primary.xml.gz");
  }

  private FluentAsset cached(final String path, final String algorithm, final String checksum) {
    FluentAsset asset = mock(FluentAsset.class);
    AssetBlob blob = mock(AssetBlob.class);
    when(blob.checksums()).thenReturn(Map.of(algorithm, checksum));
    when(asset.blob()).thenReturn(Optional.of(blob));
    when(content.getAsset(path)).thenReturn(Optional.of(asset));
    return asset;
  }

  private static RepomdData data(final String type, final String checksumType, final String checksum, final String href) {
    return new RepomdData(type, checksumType, checksum, null, null, href, 1L, 1L, null, null);
  }

  private static Content repomd(final List<RepomdData> data) throws Exception {
    return new Content(new BytesPayload(new Repomd("1", data).toXml(), "application/xml"));
  }
}
