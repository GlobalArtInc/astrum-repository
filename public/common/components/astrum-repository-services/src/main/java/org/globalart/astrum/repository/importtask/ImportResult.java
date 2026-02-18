
package org.globalart.astrum.repository.importtask;

import java.io.File;
import java.util.List;

import org.globalart.astrum.repository.Repository;

/**
 * Includes data that were going through the import process.
 */
public class ImportResult
{
  private final Repository repository;

  private final List<FileAsset> fileAssets;

  public ImportResult(
      final Repository repository,
      final List<FileAsset> fileAssets)
  {
    this.repository = repository;
    this.fileAssets = fileAssets;
  }

  public Repository getRepository() {
    return repository;
  }

  public List<FileAsset> getFileAssets() {
    return fileAssets;
  }

  public static class FileAsset
  {
    private final File file;

    private final String assetName;

    public FileAsset(
        final File file,
        final String assetName)
    {
      this.file = file;
      this.assetName = assetName;
    }

    public File getFile() {
      return file;
    }

    public String getAssetName() {
      return assetName;
    }
  }
}
