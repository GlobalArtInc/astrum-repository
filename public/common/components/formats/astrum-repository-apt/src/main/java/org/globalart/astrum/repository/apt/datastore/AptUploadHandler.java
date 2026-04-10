
package org.globalart.astrum.repository.apt.datastore;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Collections;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.AptUploadHandlerSupport;
import org.globalart.astrum.repository.apt.datastore.internal.hosted.AptHostedFacet;
import org.globalart.astrum.repository.apt.internal.AptFacetHelper;
import org.globalart.astrum.repository.apt.internal.AptPackageParser;
import org.globalart.astrum.repository.apt.internal.debian.ControlFile;
import org.globalart.astrum.repository.apt.internal.debian.PackageInfo;
import org.globalart.astrum.repository.importtask.ImportFileConfiguration;
import org.globalart.astrum.repository.importtask.ImportStreamConfiguration;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.PathPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.view.payloads.TempBlobPayload;

import org.springframework.beans.factory.annotation.Qualifier;

import static org.apache.commons.lang3.StringUtils.prependIfMissing;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Support for uploading an Apt components via UI
 *
 * @since 3.31
 */
@Qualifier(AptFormat.NAME)
@Component
public class AptUploadHandler
    extends AptUploadHandlerSupport
{
  @Autowired
  public AptUploadHandler(
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(variableResolverAdapter, contentPermissionChecker, uploadDefinitionExtensions);
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    AptContentFacet aptContentFacet = repository.facet(AptContentFacet.class);
    AptHostedFacet hostedFacet = repository.facet(AptHostedFacet.class);
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    try (TempBlob tempBlob = aptContentFacet.getTempBlob(payload)) {
      ControlFile controlFile = AptPackageParser
          .parsePackageInfo(tempBlob)
          .getControlFile();
      String assetPath = AptFacetHelper.buildAssetPath(controlFile);
      doValidation(repository, prependIfMissing(assetPath, "/"));
      Content content = hostedFacet
          .put(assetPath, payload, new PackageInfo(controlFile))
          .markAsCached(payload)
          .download();
      return new UploadResponse(Collections.singletonList(content), Collections.singletonList(assetPath));
    }
  }

  @Override
  public Content handle(final Repository repository, final File file, final String path) throws IOException {
    return handle(new ImportFileConfiguration(repository, file, path));
  }

  @Override
  public Content handle(final ImportFileConfiguration configuration) throws IOException {
    Repository repository = configuration.getRepository();
    File file = configuration.getFile();
    AptContentFacet aptContentFacet = repository.facet(AptContentFacet.class);
    AptHostedFacet hostedFacet = repository.facet(AptHostedFacet.class);

    Payload payload = new PathPayload(file.toPath(), Files.probeContentType(file.toPath()));
    try (TempBlob tempBlob = aptContentFacet.getTempBlob(payload)) {
      ControlFile controlFile = AptPackageParser
          .parsePackageInfo(tempBlob)
          .getControlFile();
      String assetPath = AptFacetHelper.buildAssetPath(controlFile);
      doValidation(repository, prependIfMissing(assetPath, "/"));
      return hostedFacet
          .put(assetPath, payload, new PackageInfo(controlFile))
          .markAsCached(payload)
          .download();
    }
  }

  @Override
  public Content handle(final ImportStreamConfiguration configuration) throws IOException {
    Repository repository = configuration.getRepository();
    AptContentFacet aptContentFacet = repository.facet(AptContentFacet.class);
    AptHostedFacet hostedFacet = repository.facet(AptHostedFacet.class);

    try (TempBlob tempBlob = aptContentFacet.getTempBlob(configuration.getInputStream(), null)) {
      ControlFile controlFile = AptPackageParser
          .parsePackageInfo(tempBlob)
          .getControlFile();
      String assetPath = AptFacetHelper.buildAssetPath(controlFile);
      doValidation(repository, prependIfMissing(assetPath, "/"));
      Payload payload = new TempBlobPayload(tempBlob, null);
      return hostedFacet
          .put(assetPath, payload, new PackageInfo(controlFile))
          .markAsCached(payload)
          .download();
    }
  }

  @Override
  public boolean supportsExportImport() {
    return true;
  }
}
