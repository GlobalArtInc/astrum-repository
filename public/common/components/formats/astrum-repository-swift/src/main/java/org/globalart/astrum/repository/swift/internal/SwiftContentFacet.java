package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.RepositoryContent;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.fasterxml.jackson.databind.node.ObjectNode;

@Facet.Exposed
public interface SwiftContentFacet
    extends ContentFacet
{
  String ATTRIBUTES = "swift";

  String CHECKSUM = "checksum";

  String PUBLISHED_AT = "publishedAt";

  String METADATA = "metadata";

  String SIGNING = "signing";

  String METADATA_SIGNING = "metadataSigning";

  String DESCRIPTION = "description";

  String REPOSITORY_URLS = "repositoryURLs";

  String TOOLS_VERSION = "toolsVersion";

  String ALTERNATES = "alternates";

  String LOCATIONS = "locations";

  SwiftCoordinates canonical(SwiftCoordinates coordinates);

  List<FluentComponent> components(SwiftCoordinates coordinates);

  Optional<FluentComponent> release(SwiftCoordinates coordinates);

  List<FluentComponent> releases(SwiftCoordinates coordinates);

  Optional<FluentAsset> getAsset(String path);

  TempBlob ingest(InputStream in, String contentType);

  FluentComponent publish(
      SwiftCoordinates coordinates,
      TempBlob archive,
      @Nullable byte[] metadata,
      @Nullable Signing signing,
      @Nullable Signing metadataSigning) throws IOException;

  List<String> identifiers(String url);

  FluentAsset storeReleases(SwiftCoordinates coordinates, Content content, List<Map<String, String>> locations);

  FluentAsset storeRelease(SwiftCoordinates coordinates, Content content) throws IOException;

  FluentAsset storeManifest(
      SwiftCoordinates coordinates,
      String filename,
      Content content,
      Map<String, String> alternates) throws IOException;

  FluentAsset storeArchive(SwiftCoordinates coordinates, Content content) throws IOException;

  @SuppressWarnings("unchecked")
  static Map<String, Object> swiftAttributes(final RepositoryContent content) {
    Object value = content.attributes().get(ATTRIBUTES);
    return value instanceof Map<?, ?> map ? (Map<String, Object>) map : Map.of();
  }

  @Nullable
  static Signing signing(@Nullable final Object value) {
    if (value instanceof Map<?, ?> map && map.get("signatureBase64Encoded") instanceof String signature
        && map.get("signatureFormat") instanceof String format) {
      return new Signing(signature, format);
    }
    return null;
  }

  static ObjectNode metadata(final RepositoryContent component) {
    return ReleaseMetadata.toNode(swiftAttributes(component).get(METADATA));
  }
}
