package org.globalart.astrum.repository.swift.internal.upload;

import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.swift.internal.SwiftContentFacet;
import org.globalart.astrum.repository.swift.internal.SwiftProblem;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition.Type;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.lang3.StringUtils.trimToNull;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_ZIP;

@Component
@Qualifier(SwiftFormat.NAME)
public class SwiftUploadHandler
    extends UploadHandlerSupport
{
  static final String SCOPE = "scope";

  static final String NAME = "name";

  static final String VERSION = "version";

  private static final String FIELD_GROUP_NAME = "Package";

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public SwiftUploadHandler(
      final ContentPermissionChecker contentPermissionChecker,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final Set<UploadDefinitionExtension> uploadDefinitionExtensions)
  {
    super(uploadDefinitionExtensions);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
  }

  @Override
  public UploadResponse handle(final Repository repository, final ComponentUpload upload) throws IOException {
    SwiftCoordinates coordinates = coordinates(upload.getFields());
    ensurePermitted(repository.getName(), SwiftFormat.NAME, coordinates.getArchivePath().substring(1),
        Collections.emptyMap());
    SwiftContentFacet content = repository.facet(SwiftContentFacet.class);
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    try (InputStream in = payload.openInputStream(); TempBlob archive = content.ingest(in, APPLICATION_ZIP)) {
      FluentComponent component = content.publish(coordinates, archive, null, null, null);
      SwiftCoordinates canonical = coordinates.withPackage(component.namespace(), component.name());
      return new UploadResponse(
          Collections.singletonList(content.getAsset(canonical.getArchivePath()).orElseThrow().download()),
          Collections.singletonList(canonical.getArchivePath()));
    }
    catch (SwiftProblem e) {
      throw new ValidationErrorsException(e.getMessage());
    }
  }

  static SwiftCoordinates coordinates(final Map<String, String> fields) {
    String scope = trimToNull(fields.get(SCOPE));
    String name = trimToNull(fields.get(NAME));
    String version = trimToNull(fields.get(VERSION));
    if (!SwiftCoordinates.isValidScope(scope)) {
      throw new ValidationErrorsException(SCOPE, "Scope must be 1 to 39 letters, digits or single inner hyphens");
    }
    if (!SwiftCoordinates.isValidName(name)) {
      throw new ValidationErrorsException(NAME,
          "Name must be 1 to 100 letters, digits or single inner hyphens and underscores");
    }
    if (!SwiftVersion.isValid(version)) {
      throw new ValidationErrorsException(VERSION, "Version must be a semantic version such as 1.0.0");
    }
    return new SwiftCoordinates(scope, name, version);
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(SwiftFormat.NAME, false,
          List.of(
              new UploadFieldDefinition(SCOPE, "Package scope", false, Type.STRING, FIELD_GROUP_NAME),
              new UploadFieldDefinition(NAME, "Package name", false, Type.STRING, FIELD_GROUP_NAME),
              new UploadFieldDefinition(VERSION, "Release version", false, Type.STRING, FIELD_GROUP_NAME)),
          Collections.emptyList(),
          null);
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
