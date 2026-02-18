
package org.globalart.astrum.repository.importtask;

import java.io.InputStream;
import java.util.Objects;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Hold the information needed for importing from an InputStream
 *
 * @since 3.31
 */
public class ImportStreamConfiguration
{
  private final Repository repository;

  private final InputStream inputStream;

  private final String assetName;

  private final String contentType;

  /**
   * @param repository the {@link Repository} to add the stream content to
   * @param inputStream the {@link InputStream} containing the content to add to the repository
   * @param assetName the path of the content relative to the base import directory
   */
  public ImportStreamConfiguration(final Repository repository, final InputStream inputStream, final String assetName) {
    this(repository, inputStream, assetName, null);
  }

  /**
   * @param repository the {@link Repository} to add the stream content to
   * @param inputStream the {@link InputStream} containing the content to add to the repository
   * @param assetName the path of the content relative to the base import directory
   * @param contentType the content type of the asset (may be null for auto-detection)
   */
  public ImportStreamConfiguration(
      final Repository repository,
      final InputStream inputStream,
      final String assetName,
      @Nullable final String contentType)
  {
    this.repository = checkNotNull(repository);
    this.inputStream = checkNotNull(inputStream);
    this.assetName = checkNotNull(assetName);
    this.contentType = contentType;
  }

  public Repository getRepository() {
    return repository;
  }

  public InputStream getInputStream() {
    return inputStream;
  }

  public String getAssetName() {
    return assetName;
  }

  @Nullable
  public String getContentType() {
    return contentType;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ImportStreamConfiguration that = (ImportStreamConfiguration) o;
    return Objects.equals(repository, that.repository) &&
        Objects.equals(inputStream, that.inputStream) &&
        Objects.equals(assetName, that.assetName) &&
        Objects.equals(contentType, that.contentType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(repository, inputStream, assetName, contentType);
  }
}
