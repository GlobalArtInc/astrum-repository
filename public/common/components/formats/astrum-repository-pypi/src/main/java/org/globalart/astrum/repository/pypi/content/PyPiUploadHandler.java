
package org.globalart.astrum.repository.pypi.content;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Collections;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.pypi.internal.PyPiAttributes;
import org.globalart.astrum.repository.pypi.internal.PyPiFileUtils;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.content.PyPiHostedFacet;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.StreamPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.view.payloads.TempBlobPartPayload;
import org.globalart.astrum.rest.ValidationErrorsException;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.apache.commons.lang3.StringUtils.isBlank;
import static org.apache.commons.lang3.StringUtils.isNotBlank;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_ARCHIVE_TYPE;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiUploadHandler
    extends UploadHandlerSupport
{
  private UploadDefinition definition;

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  @Autowired
  public PyPiUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.contentPermissionChecker = contentPermissionChecker;
    this.variableResolverAdapter = variableResolverAdapter;
  }

  @Override
  public Content handle(final Repository repository, final File content, final String path) throws IOException {
    if (content.getName().endsWith(".asc")) {
      return importSignatureFile(content, path, repository);
    }
    return importPackageFile(content, repository);
  }

  private Content importPackageFile(final File packageFile, final Repository repository) throws IOException {
    PyPiHostedFacet facet = repository.facet(PyPiHostedFacet.class);
    PypiContentFacet contentFacet = repository.facet(PypiContentFacet.class);

    String contentType = Files.probeContentType(packageFile.toPath());
    Payload payload = new StreamPayload(() -> new FileInputStream(packageFile), packageFile.length(), contentType);

    try (TempBlob tempBlob = contentFacet.getTempBlob(payload)) {
      Map<String, String> metadata = getAndValidateMetadata(repository, tempBlob);
      String name = getAndValidateName(metadata);
      String version = getAndValidateVersion(metadata);
      String filename = packageFile.getName();
      String packagePath = facet.createPackagePath(name, version, filename);

      ensurePermitted(repository.getName(), PyPiFormat.NAME, packagePath, coordinatesFromMetadata(metadata));
      facet.upload(filename, metadata, new TempBlobPartPayload("", false, filename, contentType, tempBlob));
      return facet.getPackage(packagePath);
    }
  }

  private Content importSignatureFile(final File signatureFile, final String path, final Repository repository)
      throws IOException
  {
    PyPiHostedFacet facet = repository.facet(PyPiHostedFacet.class);
    PypiContentFacet contentFacet = repository.facet(PypiContentFacet.class);

    Payload payload = new StreamPayload(() -> new FileInputStream(signatureFile), signatureFile.length(), TEXT_PLAIN);

    try (TempBlob tempBlob = contentFacet.getTempBlob(payload)) {
      String name = PyPiFileUtils.extractNameFromPath(path);
      String version = PyPiFileUtils.extractVersionFromPath(path);

      ensurePermitted(repository.getName(), PyPiFormat.NAME, path, ImmutableMap.of("name", name, "version", version));

      return facet.uploadSignature(name, version,
          new TempBlobPartPayload("", false, signatureFile.getName(), TEXT_PLAIN, tempBlob));
    }
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    PyPiHostedFacet facet = repository.facet(PyPiHostedFacet.class);
    PypiContentFacet contentFacet = repository.facet(PypiContentFacet.class);

    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    try (TempBlob tempBlob = contentFacet.getTempBlob(payload)) {
      Map<String, String> metadata = getAndValidateMetadata(repository, tempBlob);
      String name = getAndValidateName(metadata);
      String version = getAndValidateVersion(metadata);

      String filename = isNotBlank(payload.getName()) ? payload.getName() :
          name + '-' + version + '.' + metadata.get(P_ARCHIVE_TYPE);
      String path = facet.createPackagePath(name, version, filename);

      ensurePermitted(repository.getName(), PyPiFormat.NAME, path, coordinatesFromMetadata(metadata));

      Content content = facet.upload(filename, metadata, new TempBlobPartPayload(payload, tempBlob)).download();
      return new UploadResponse(Collections.singletonList(content), Collections.singletonList(path));
    }
  }

  private Map<String, String> getAndValidateMetadata(final Repository repository, final TempBlob tempBlob)
      throws IOException
  {
    Map<String, String> metadata = repository.facet(PyPiHostedFacet.class).extractMetadata(tempBlob);
    if (metadata.isEmpty()) {
      throw new ValidationErrorsException("Unable to extract PyPI metadata from provided archive.");
    }
    return metadata;
  }

  private String getAndValidateName(final Map<String, String> metadata) {
    String name = metadata.get(PyPiAttributes.P_NAME);
    if (isBlank(name)) {
      throw new ValidationErrorsException("Metadata is missing the name attribute");
    }
    return name;
  }

  private String getAndValidateVersion(final Map<String, String> metadata) {
    String version = metadata.get(PyPiAttributes.P_VERSION);
    if (isBlank(version)) {
      throw new ValidationErrorsException("Metadata is missing the version attribute");
    }
    return version;
  }

  private Map<String, String> coordinatesFromMetadata(final Map<String, String> packageMetadata) {
    return ImmutableMap.of(
        PyPiAttributes.P_NAME, packageMetadata.get(PyPiAttributes.P_NAME),
        PyPiAttributes.P_VERSION, packageMetadata.get(PyPiAttributes.P_VERSION));
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(PyPiFormat.NAME, false);
    }
    return definition;
  }

  @Override
  public VariableResolverAdapter getVariableResolverAdapter() {
    return variableResolverAdapter;
  }

  @Override
  public ContentPermissionChecker contentPermissionChecker() {
    return contentPermissionChecker;
  }
}
