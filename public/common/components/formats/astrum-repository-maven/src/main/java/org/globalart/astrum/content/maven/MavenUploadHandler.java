
package org.globalart.astrum.content.maven;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.importtask.ImportFileConfiguration;
import org.globalart.astrum.repository.importtask.ImportStreamConfiguration;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.maven.MavenPath.HashType;
import org.globalart.astrum.repository.maven.MavenUploadHandlerSupport;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.Maven2MavenPathParser;
import org.globalart.astrum.repository.maven.internal.MavenPomGenerator;
import org.globalart.astrum.repository.maven.internal.VersionPolicyValidator;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.view.payloads.TempBlobPayload;
import org.globalart.astrum.mime.MimeSupport;

import org.joda.time.DateTime;

import static java.util.stream.Collectors.toList;
import static org.apache.commons.lang3.StringUtils.prependIfMissing;
import static org.globalart.astrum.repository.maven.internal.Constants.CHECKSUM_CONTENT_TYPE;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Support for uploading maven components via UI & API
 *
 * @since 3.26
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenUploadHandler
    extends MavenUploadHandlerSupport
{
  private final MimeSupport mimeSupport;

  @Autowired
  public MavenUploadHandler(
      final Maven2MavenPathParser parser,
      @Qualifier(Maven2Format.NAME) final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker,
      final VersionPolicyValidator versionPolicyValidator,
      final MavenPomGenerator mavenPomGenerator,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions,
      final MimeSupport mimeSupport)
  {
    super(parser, variableResolverAdapter, contentPermissionChecker, versionPolicyValidator, mavenPomGenerator,
        uploadDefinitionExtensions, true);
    this.mimeSupport = mimeSupport;
  }

  @Override
  protected UploadResponse getUploadResponse(
      final Repository repository,
      final ComponentUpload componentUpload,
      final String basePath) throws IOException
  {
    ContentAndAssetPathResponseData responseData =
        createAssets(repository, basePath, componentUpload.getAssetUploads());
    maybeGeneratePom(repository, componentUpload, basePath, responseData);
    updateMetadata(repository, responseData.getCoordinates());
    return new UploadResponse(responseData.getContent(), responseData.getAssetPaths()
        .stream()
        .map(assetPath -> prependIfMissing(assetPath, "/"))
        .collect(toList()));
  }

  private void updateMetadata(final Repository repository, final Coordinates coordinates) {
    if (coordinates != null) {
      repository.facet(MavenMetadataRebuildFacet.class)
          .rebuildMetadata(coordinates.getGroupId(), coordinates.getArtifactId(), coordinates.getVersion(),
              false, false);
    }
    else {
      log.debug("Not updating metadata.xml files since coordinate could not be retrieved from path");
    }
  }

  @Override
  protected Content doPut(final ImportFileConfiguration configuration) throws IOException {
    Repository repository = configuration.getRepository();
    MavenPath mavenPath = parser.parsePath(configuration.getAssetName());
    File content = configuration.getFile();
    Path contentPath = content.toPath();

    MavenContentFacet contentFacet = repository.facet(MavenContentFacet.class);
    String contentType = Files.probeContentType(contentPath);
    try (TempBlob blob = contentFacet.blobs()
        .ingest(contentPath, contentType, MavenPath.HashType.ALGORITHMS,
            configuration.isHardLinkingEnabled())) {
      return doPut(repository, mavenPath, new TempBlobPayload(blob, contentType));
    }
  }

  @Override
  protected Content doPut(final ImportStreamConfiguration configuration) throws IOException {
    Repository repository = configuration.getRepository();
    MavenPath mavenPath = parser.parsePath(configuration.getAssetName());

    MavenContentFacet contentFacet = repository.facet(MavenContentFacet.class);
    // Use the same approach as Files.probeContentType but with fallback to extension-based detection
    String contentType = determineContentTypeFromPath(configuration.getAssetName());
    try (TempBlob blob = contentFacet.blobs()
        .ingest(configuration.getInputStream(), null, MavenPath.HashType.ALGORITHMS)) {
      // Create TempBlobPayload with explicit content type to override any detected type
      return doPut(repository, mavenPath, new TempBlobPayload(blob, contentType));
    }
  }

  protected String determineContentTypeFromPath(String assetName) {
    // First try Java's built-in content type detection
    try {
      String detectedType = Files.probeContentType(java.nio.file.Paths.get(assetName));
      if (detectedType != null) {
        return detectedType;
      }
    }
    catch (Exception e) {
      // Fall through to extension-based detection
    }

    // Use MimeSupport for comprehensive MIME type detection
    return mimeSupport.guessMimeTypeFromPath(assetName);
  }

  @Override
  protected Content doPut(
      final Repository repository,
      final MavenPath mavenPath,
      final Payload payload) throws IOException
  {
    MavenContentFacet mavenFacet = repository.facet(MavenContentFacet.class);
    Content asset = mavenFacet.put(mavenPath, payload);
    if (!mavenPath.isHash()) {
      putChecksumFiles(mavenFacet, mavenPath, asset);
    }
    return asset;
  }

  @Override
  protected VersionPolicy getVersionPolicy(final Repository repository) {
    return repository.facet(MavenContentFacet.class).getVersionPolicy();
  }

  @Override
  protected TempBlob createTempBlob(final Repository repository, final PartPayload payload) {
    return repository.facet(MavenContentFacet.class).blobs().ingest(payload, MavenPath.HashType.ALGORITHMS);
  }

  private void putChecksumFiles(
      final MavenContentFacet facet,
      final MavenPath path,
      final Content content) throws IOException
  {
    DateTime dateTime = content.getAttributes().require(CONTENT_LAST_MODIFIED, DateTime.class);
    for (Entry<String, String> e : getChecksumsFromContent(content).entrySet()) {
      Optional<HashAlgorithm> hashAlgorithm = HashAlgorithm.getHashAlgorithm(e.getKey())
          .filter(HashType.ALGORITHMS::contains);
      if (hashAlgorithm.isPresent()) {
        Content c = new Content(new StringPayload(e.getValue(), CHECKSUM_CONTENT_TYPE));
        c.getAttributes().set(CONTENT_LAST_MODIFIED, dateTime);
        facet.put(path.hash(HashType.valueOf(e.getKey().toUpperCase())), c);
      }
    }
  }

  private Map<String, String> getChecksumsFromContent(final Content content) {
    return Optional.ofNullable(content.getAttributes().get(Asset.class))
        .flatMap(Asset::blob)
        .map(AssetBlob::checksums)
        .orElse(Collections.emptyMap());
  }
}
