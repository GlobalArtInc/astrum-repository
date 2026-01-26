
package org.globalart.astrum.blobstore.file.internal;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.blobstore.BlobStoreDescriptor;
import org.globalart.astrum.blobstore.BlobStoreDescriptorSupport;
import org.globalart.astrum.blobstore.BlobStoreUtil;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.rest.ValidationErrorsException;

import static java.lang.String.format;
import static java.util.Arrays.asList;
import static org.globalart.astrum.blobstore.file.FileBlobStore.BASEDIR;
import static org.globalart.astrum.blobstore.file.FileBlobStore.CONFIG_KEY;
import static org.globalart.astrum.blobstore.file.FileBlobStore.PATH_KEY;
import static org.globalart.astrum.formfields.FormField.MANDATORY;
import static org.globalart.astrum.blobstore.BlobStoreSupport.MAX_NAME_LENGTH;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * A {@link BlobStoreDescriptor} for {@link FileBlobStore}.
 *
 * @since 3.6
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(FileBlobStore.TYPE)
public class FileBlobStoreDescriptor
    extends BlobStoreDescriptorSupport
{

  public static final String PATH_HELP_TEXT =
      "An absolute path or a path relative to <data-directory>/blobs";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("File")
    String name();

    @DefaultMessage("Path")
    String pathLabel();

    @DefaultMessage(PATH_HELP_TEXT)
    String pathHelpText();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final ApplicationDirectories applicationDirectories;

  private final FormField<String> path;

  private final BlobStoreUtil blobStoreUtil;

  private final FileBlobStorePathValidator pathValidator;

  @Autowired
  public FileBlobStoreDescriptor(
      final BlobStoreQuotaService quotaService,
      final ApplicationDirectories applicationDirectories,
      final BlobStoreUtil blobStoreUtil,
      final FileBlobStorePathValidator pathValidator)
  {
    super(quotaService);
    this.applicationDirectories = applicationDirectories;
    this.blobStoreUtil = blobStoreUtil;
    this.pathValidator = pathValidator;
    this.path = new StringTextFormField(PATH_KEY, messages.pathLabel(), messages.pathHelpText(), MANDATORY)
        .withAttribute("tokenReplacement", applicationDirectories.getWorkDirectory("blobs") + "/${name}")
        .withAttribute("long", Boolean.TRUE);
  }

  @Override
  public String getId() {
    return "file";
  }

  @Override
  public String getName() {
    return messages.name();
  }

  @Override
  public List<FormField> getFormFields() {
    return asList(path);
  }

  @Override
  public void validateConfig(final BlobStoreConfiguration config) {
    super.validateConfig(config);

    Path path = Optional.ofNullable(config.attributes(CONFIG_KEY).get(PATH_KEY, String.class))
        .filter(s -> blobStoreUtil.validateFilePath(s, MAX_NAME_LENGTH))
        .map(s -> Paths.get(s))
        .orElseThrow(() -> new ValidationErrorsException(
            format("The maximum name length for any folder in the path is %d.", MAX_NAME_LENGTH)));

    pathValidator.validatePathUniqueConstraint(config);

    try {
      if (!path.isAbsolute()) {
        Path baseDir = applicationDirectories.getWorkDirectory(BASEDIR).toPath().toRealPath().normalize();
        path = baseDir.resolve(path.normalize());
      }

      if (!Files.isDirectory(path)) {
        Files.createDirectories(path);
      }
      else if (Files.isRegularFile(path)) {
        throw new ValidationErrorsException(
            format("Blob store could not be written to path '%s' because it is a file not a directory", path));
      }
    }
    catch (IOException e) {
      throw new ValidationErrorsException(
          format("Blob store could not be written because the path %s could not be written to", path), e);
    }

    if (!Files.isWritable(path)) {
      throw new ValidationErrorsException(
          format("Blob store could not be written to path '%s' because it was not writable", path));
    }
  }
}
