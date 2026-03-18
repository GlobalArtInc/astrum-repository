
package org.globalart.astrum.repository.raw;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.importtask.ImportFileConfiguration;
import org.globalart.astrum.repository.importtask.ImportStreamConfiguration;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition.Type;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadRegexMap;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;

import static java.util.Collections.emptyMap;
import static java.util.Collections.singletonList;
import static org.apache.commons.lang3.StringUtils.prependIfMissing;

/**
 * Common base for raw upload handlers
 *
 * @since 3.24
 */
public abstract class RawUploadHandlerSupport
    extends UploadHandlerSupport
{
  protected static final String FILENAME = "filename";

  protected static final String DIRECTORY = "directory";

  protected static final String DIRECTORY_HELP_TEXT = "Destination for uploaded files (e.g. /path/to/files/)";

  protected static final String FIELD_GROUP_NAME = "Component attributes";

  protected final ContentPermissionChecker contentPermissionChecker;

  protected final VariableResolverAdapter variableResolverAdapter;

  protected final boolean datastoreEnabled;

  protected UploadDefinition definition;

  public RawUploadHandlerSupport(
      final ContentPermissionChecker contentPermissionChecker,
      final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions,
      final boolean datastoreEnabled)
  {
    super(uploadDefinitionExtensions);
    this.contentPermissionChecker = contentPermissionChecker;
    this.variableResolverAdapter = variableResolverAdapter;
    this.datastoreEnabled = datastoreEnabled;
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    String basePath = upload.getFields().get(DIRECTORY).trim();

    // Data holders for populating the UploadResponse
    Map<String, PartPayload> pathToPayload = new LinkedHashMap<>();

    for (AssetUpload asset : upload.getAssetUploads()) {
      String path = normalizePath(basePath + '/' + asset.getFields().get(FILENAME).trim());

      String pathWithPrefix = datastoreEnabled ? prependIfMissing(path, "/") : path;
      ensurePermitted(repository.getName(), RawFormat.NAME, pathWithPrefix, emptyMap());

      pathToPayload.put(path, asset.getPayload());
    }

    List<Content> responseContents = getResponseContents(repository, pathToPayload);

    return new UploadResponse(responseContents, new ArrayList<>(pathToPayload.keySet()));
  }

  protected abstract List<Content> getResponseContents(
      final Repository repository,
      final Map<String, PartPayload> pathToPayload) throws IOException;

  @Override
  public Content handle(
      final Repository repository,
      final File content,
      final String path) throws IOException
  {
    // TODO: Remove this handler once all formats have been converted to work with ImportFileConfiguration
    return handle(new ImportFileConfiguration(repository, content, path));
  }

  @Override
  public Content handle(final ImportFileConfiguration configuration) throws IOException {

    ensurePermitted(configuration.getRepository().getName(), RawFormat.NAME, configuration.getAssetName(), emptyMap());

    return doPut(configuration);
  }

  @Override
  public Content handle(final ImportStreamConfiguration configuration) throws IOException {

    ensurePermitted(configuration.getRepository().getName(), RawFormat.NAME, configuration.getAssetName(), emptyMap());

    return doPut(configuration);
  }

  protected abstract Content doPut(final ImportFileConfiguration configuration) throws IOException;

  protected abstract Content doPut(final ImportStreamConfiguration configuration) throws IOException;

  protected String normalizePath(final String path) {
    String result = path.replace('\\', '/').replaceAll("/+", "/");

    if (result.startsWith("/")) {
      result = result.substring(1);
    }

    if (result.endsWith("/")) {
      result = result.substring(0, result.length() - 1);
    }

    return result;
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(RawFormat.NAME, true,
          singletonList(
              new UploadFieldDefinition(DIRECTORY, DIRECTORY_HELP_TEXT, false, Type.STRING, FIELD_GROUP_NAME)),
          singletonList(new UploadFieldDefinition(FILENAME, false, Type.STRING)),
          new UploadRegexMap("(.*)", FILENAME));
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

  @Override
  public boolean supportsExportImport() {
    return true;
  }
}
