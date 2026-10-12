package org.globalart.astrum.repository.terraform.internal.hosted;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition.Type;
import org.globalart.astrum.repository.upload.UploadHandlerSupport;
import org.globalart.astrum.repository.upload.UploadResponse;
import org.globalart.astrum.repository.upload.ValidatingComponentUpload;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Uploads through the components API: a module package with {@code namespace}, {@code name}, {@code system} and
 * {@code version}, or the release files of a provider version with {@code namespace}, {@code type} and
 * {@code version}, several files at once. Provider files are recognized by their names, as release tooling writes
 * them.
 */
@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformUploadHandler
    extends UploadHandlerSupport
{
  public static final String UPLOAD_TYPE = "uploadType";

  public static final String MODULE = "module";

  public static final String PROVIDER = "provider";

  public static final String NAMESPACE = "namespace";

  public static final String NAME = "name";

  public static final String SYSTEM = "system";

  public static final String TYPE = "type";

  public static final String VERSION = "version";

  public static final String PROTOCOLS = "protocols";

  static final String COORDINATES_GROUP = "Coordinates";

  static final String MODULE_GROUP = "Module Coordinates";

  static final String PROVIDER_GROUP = "Provider Coordinates";

  private final ContentPermissionChecker contentPermissionChecker;

  private final VariableResolverAdapter variableResolverAdapter;

  private UploadDefinition definition;

  @Autowired
  public TerraformUploadHandler(
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
    Map<String, String> fields = upload.getFields();
    String uploadType = uploadType(fields);
    TerraformHostedFacet hosted = repository.facet(TerraformHostedFacet.class);
    List<Content> contents = new ArrayList<>();
    try {
      if (MODULE.equals(uploadType)) {
        contents.add(putModule(repository, hosted, fields, upload));
      }
      else {
        contents.addAll(putProviderFiles(repository, hosted, fields, upload));
      }
    }
    catch (InvalidContentException e) {
      throw new ValidationErrorsException(e.getMessage());
    }
    List<String> paths = contents.stream()
        .map(content -> content.getAttributes().require(Asset.class).path())
        .toList();
    return new UploadResponse(contents, paths);
  }

  private Content putModule(
      final Repository repository,
      final TerraformHostedFacet hosted,
      final Map<String, String> fields,
      final ComponentUpload upload) throws IOException
  {
    if (upload.getAssetUploads().size() != 1) {
      throw new ValidationErrorsException("A module upload takes exactly one package");
    }
    ModuleCoordinates coordinates = new ModuleCoordinates(required(fields, NAMESPACE), required(fields, NAME),
        required(fields, SYSTEM), requiredVersion(fields));
    PartPayload payload = upload.getAssetUploads().get(0).getPayload();
    return hosted.putModule(coordinates, payload.getName(), payload,
        path -> ensurePermitted(repository.getName(), TerraformFormat.NAME, path, Map.of()));
  }

  /**
   * Stores the release files of one provider version, recognized by their names; the hosted facet checks them with
   * each other before it stores any.
   */
  private List<Content> putProviderFiles(
      final Repository repository,
      final TerraformHostedFacet hosted,
      final Map<String, String> fields,
      final ComponentUpload upload) throws IOException
  {
    ProviderCoordinates coordinates = new ProviderCoordinates(required(fields, NAMESPACE), required(fields, TYPE),
        requiredVersion(fields));
    List<String> protocols = protocols(fields.get(PROTOCOLS));
    Map<String, PartPayload> files = new LinkedHashMap<>();
    for (AssetUpload asset : upload.getAssetUploads()) {
      PartPayload payload = asset.getPayload();
      String fileName = payload.getName();
      if (fileName == null || fileName.isBlank()) {
        throw new ValidationErrorsException("Provider files are recognized by their names; a file has no name");
      }
      if (files.put(baseName(fileName), payload) != null) {
        throw new ValidationErrorsException("The upload holds " + baseName(fileName) + " more than once");
      }
    }
    return hosted.putProviderFiles(coordinates, files, protocols,
        path -> ensurePermitted(repository.getName(), TerraformFormat.NAME, path, Map.of()));
  }

  /**
   * Validates the fields only: provider files carry no asset fields, so several of them look alike to the generic
   * duplicate check.
   */
  @Override
  public ValidatingComponentUpload getValidatingComponentUpload(final ComponentUpload componentUpload) {
    return new ValidatingComponentUpload(getDefinition(), componentUpload)
    {
      @Override
      public ComponentUpload getComponentUpload() {
        ValidationErrorsException errors = new ValidationErrorsException();
        if (componentUpload.getAssetUploads().isEmpty()) {
          errors.withError("No assets found in upload");
        }
        Set<String> allowed = Set.of(UPLOAD_TYPE, NAMESPACE, NAME, SYSTEM, TYPE, VERSION, PROTOCOLS);
        componentUpload.getFields().keySet().stream()
            .filter(field -> !allowed.contains(field))
            .forEach(field -> errors.withError(field, "Unknown component field '" + field + "'"));
        for (int i = 0; i < componentUpload.getAssetUploads().size(); i++) {
          AssetUpload asset = componentUpload.getAssetUploads().get(i);
          if (asset.getPayload() == null) {
            errors.withError("file", "Missing file on asset '" + (i + 1) + "'");
          }
          if (asset.getFields() != null && !asset.getFields().isEmpty()) {
            errors.withError("Provider and module files take no asset fields");
          }
        }
        if (!errors.getValidationErrors().isEmpty()) {
          throw errors;
        }
        return componentUpload;
      }
    };
  }

  @Override
  public UploadDefinition getDefinition() {
    if (definition == null) {
      definition = getDefinition(TerraformFormat.NAME, true, List.of(
          new UploadFieldDefinition(UPLOAD_TYPE, "Upload Type",
              "module or provider; taken from the coordinates when empty", true, Type.STRING, COORDINATES_GROUP),
          new UploadFieldDefinition(NAMESPACE, "Namespace", "Organization or group of the module or provider",
              false, Type.STRING, COORDINATES_GROUP),
          new UploadFieldDefinition(VERSION, "Version", "Semantic version, for example 1.2.0", false, Type.STRING,
              COORDINATES_GROUP),
          new UploadFieldDefinition(NAME, "Name", "Module name, for example vpc", true, Type.STRING, MODULE_GROUP),
          new UploadFieldDefinition(SYSTEM, "System", "Target system of the module, for example aws", true,
              Type.STRING, MODULE_GROUP),
          new UploadFieldDefinition(TYPE, "Type", "Provider type, for example aws", true, Type.STRING,
              PROVIDER_GROUP),
          new UploadFieldDefinition(PROTOCOLS, "Protocols",
              "Plugin protocol versions of the provider, for example 6.0; 5.0 when empty", true, Type.STRING,
              PROVIDER_GROUP)),
          List.of(), null);
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

  static String uploadType(final Map<String, String> fields) {
    String declared = fields.get(UPLOAD_TYPE);
    if (declared != null && !declared.isBlank()) {
      String value = declared.strip().toLowerCase(Locale.ROOT);
      if (!MODULE.equals(value) && !PROVIDER.equals(value)) {
        throw new ValidationErrorsException(UPLOAD_TYPE, "Upload type must be module or provider");
      }
      return value;
    }
    boolean module = present(fields, SYSTEM) || present(fields, NAME);
    boolean provider = present(fields, TYPE);
    if (module == provider) {
      throw new ValidationErrorsException(UPLOAD_TYPE,
          "Give name and system for a module, or type for a provider");
    }
    return module ? MODULE : PROVIDER;
  }

  @Nullable
  static List<String> protocols(@Nullable final String value) {
    if (value == null || value.isBlank()) {
      return null;
    }
    List<String> protocols = Arrays.stream(value.split("[,\\s]+"))
        .filter(protocol -> !protocol.isBlank())
        .map(protocol -> protocol.contains(".") ? protocol : protocol + ".0")
        .toList();
    for (String protocol : protocols) {
      if (!protocol.matches("[0-9]+\\.[0-9]+")) {
        throw new ValidationErrorsException(PROTOCOLS, "Invalid plugin protocol version: " + protocol);
      }
    }
    return protocols;
  }

  private static String baseName(final String fileName) {
    String name = fileName.replace('\\', '/');
    return name.substring(name.lastIndexOf('/') + 1);
  }

  private static String required(final Map<String, String> fields, final String field) {
    String value = fields.get(field);
    if (value == null || value.isBlank()) {
      throw new ValidationErrorsException(field, "Missing required component field '" + field + "'");
    }
    String stripped = value.strip();
    if (!TerraformPaths.isValidName(stripped)) {
      throw new ValidationErrorsException(field, "Invalid " + field + " '" + stripped
          + "': use letters, digits, '-' and '_'");
    }
    return stripped;
  }

  private static String requiredVersion(final Map<String, String> fields) {
    String value = fields.get(VERSION);
    if (value == null || value.isBlank()) {
      throw new ValidationErrorsException(VERSION, "Missing required component field 'version'");
    }
    String stripped = value.strip();
    if (stripped.startsWith("v") && TerraformPaths.isValidVersion(stripped.substring(1))) {
      stripped = stripped.substring(1);
    }
    if (!TerraformPaths.isValidVersion(stripped)) {
      throw new ValidationErrorsException(VERSION, "Version '" + stripped + "' is not a semantic version");
    }
    return stripped;
  }

  private static boolean present(final Map<String, String> fields, final String field) {
    String value = fields.get(field);
    return value != null && !value.isBlank();
  }
}
