package org.globalart.astrum.repository.cocoapods;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface CocoapodsContentFacet
    extends ContentFacet
{
  String POD_NAME_ATTRIBUTE = "name";

  String POD_VERSION_ATTRIBUTE = "version";

  String POD_REMOTE_ATTRIBUTE = "remote_url";

  Optional<FluentAsset> getAsset(String path);

  FluentAsset putMetadata(String path, TempBlob tempBlob);

  FluentAsset putSpec(
      String path,
      TempBlob tempBlob,
      @Nullable String name,
      @Nullable String version,
      @Nullable String remoteUrl);

  FluentAsset putPod(String path, String name, String version, String remoteUrl, TempBlob tempBlob);

  @Nullable
  String getPodRemoteUrl(String specPath);

  TempBlob getTempBlob(Payload payload);
}
