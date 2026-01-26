
package org.globalart.astrum.blobstore.file.internal.datastore.metrics;

import java.io.IOException;
import java.nio.file.DirectoryStream;
import java.nio.file.FileStore;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.BlobStoreMetricsNotAvailableException;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.metrics.BlobStoreMetricsPropertiesReaderSupport;
import org.globalart.astrum.common.property.PropertiesFile;

import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * A {@link BlobStoreMetricsPropertiesReaderSupport} implementation that retains blobstore metrics in memory,
 * periodically
 * writing them out to a file.
 *
 * @since 3.0
 * @deprecated legacy method for metrics stored in the blob store
 */
@Deprecated
@Component
@Qualifier(FileBlobStore.TYPE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class FileBlobStoreMetricsPropertiesReader
    extends BlobStoreMetricsPropertiesReaderSupport<FileBlobStore, PropertiesFile>
{
  private Path storageDirectory;

  @Override
  protected void doInit(final FileBlobStore blobstore) throws IOException {
    Path storageDir = blobStore.getAbsoluteBlobDir();

    checkNotNull(storageDir);
    checkArgument(Files.isDirectory(storageDir));
    this.storageDirectory = storageDir;
  }

  @Override
  protected AccumulatingBlobStoreMetrics getAccumulatingBlobStoreMetrics() throws BlobStoreMetricsNotAvailableException {
    try {
      FileStore fileStore = Files.getFileStore(storageDirectory);
      ImmutableMap<String, Long> availableSpace = ImmutableMap
          .of("fileStore:" + fileStore.name(), fileStore.getUsableSpace());
      return new AccumulatingBlobStoreMetrics(0, 0, availableSpace, false);
    }
    catch (IOException e) {
      throw new BlobStoreMetricsNotAvailableException(e);
    }
  }

  @Override
  protected Stream<PropertiesFile> backingFiles() throws BlobStoreMetricsNotAvailableException {
    if (storageDirectory == null) {
      return Stream.empty();
    }
    try (DirectoryStream<Path> files =
        Files.newDirectoryStream(storageDirectory, path -> path.toString().endsWith(METRICS_FILENAME))) {
      return StreamSupport.stream(files.spliterator(), false)
          .map(Path::toFile)
          .map(PropertiesFile::new)
          // we need a terminal operation since the directory stream will be closed
          .collect(toList())
          .stream();
    }
    catch (IOException | SecurityException e) {
      throw new BlobStoreMetricsNotAvailableException(e);
    }
  }
}
