
package org.globalart.astrum.blobstore.s3.internal;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletionService;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorCompletionService;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Supplier;

import org.globalart.astrum.blobstore.api.BlobStoreException;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.thread.AstrumThreadFactory;

import software.amazon.awssdk.services.s3.model.AbortMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.CompleteMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.CompletedMultipartUpload;
import software.amazon.awssdk.services.s3.model.CompletedPart;

import static com.google.common.base.Preconditions.checkArgument;
import static java.lang.String.format;

/**
 * Common class to execute parallel requests to S3 for a MultipartUpload operation
 *
 * @since 3.19
 */
public abstract class ParallelRequester
    extends StateGuardLifecycleSupport
{
  protected final int chunkSize;

  private final int parallelism;

  private final ExecutorService executorService;

  /**
   * @param chunkSize - the number of bytes to be processed in one parallel request
   * @param numberOfThreads - a non-negative integer, either 0 to indicate that number of threads should be dynamically
   *          selected based on the env, or a postive int to set a fixed number of threads
   * @param threadGroupName - a human readable name for the threads
   */
  public ParallelRequester(final int chunkSize, final int numberOfThreads, final String threadGroupName) {
    checkArgument(numberOfThreads >= 0, "Must use a non-negative parallelism");
    checkArgument(chunkSize >= 0, "Must use a non-negative chunkSize");
    this.chunkSize = chunkSize;
    this.parallelism = (numberOfThreads > 0) ? numberOfThreads : Runtime.getRuntime().availableProcessors();

    this.executorService = Executors.newFixedThreadPool(parallelism,
        new AstrumThreadFactory("s3-parallel", threadGroupName));
  }

  @Override
  protected void doStop() {
    executorService.shutdownNow();
  }

  @FunctionalInterface
  protected interface IOFunction<T, R>
  {
    R apply(T v) throws IOException;
  }

  protected void parallelRequests(
      final EncryptingS3Client s3,
      final String bucket,
      final String key,
      final Supplier<IOFunction<String, List<CompletedPart>>> operations)
  {

    final String uploadId = s3.createMultipartUpload(bucket, key).uploadId();

    CompletionService<List<CompletedPart>> completionService = new ExecutorCompletionService<>(executorService);
    try {
      for (int i = 0; i < parallelism; i++) {
        completionService.submit(() -> operations.get().apply(uploadId));
      }

      List<CompletedPart> completedParts = new ArrayList<>();
      for (int i = 0; i < parallelism; i++) {
        completedParts.addAll(completionService.take().get());
      }

      // AWS S3 requires parts to be in ascending order by part number (according the Claude Code)
      completedParts.sort(Comparator.comparingInt(CompletedPart::partNumber));

      final CompletedMultipartUpload completedUpload = CompletedMultipartUpload.builder()
          .parts(completedParts)
          .build();
      s3.completeMultipartUpload(CompleteMultipartUploadRequest.builder()
          .bucket(bucket)
          .key(key)
          .uploadId(uploadId)
          .multipartUpload(completedUpload)
          .build());
    }
    catch (InterruptedException interrupted) {
      s3.abortMultipartUpload(AbortMultipartUploadRequest.builder()
          .bucket(bucket)
          .key(key)
          .uploadId(uploadId)
          .build());
      Thread.currentThread().interrupt();
    }
    catch (CancellationException | ExecutionException ex) {
      s3.abortMultipartUpload(AbortMultipartUploadRequest.builder()
          .bucket(bucket)
          .key(key)
          .uploadId(uploadId)
          .build());
      throw new BlobStoreException(
          format("Error executing parallel requests for bucket:%s key:%s with uploadId:%s", bucket, key, uploadId), ex,
          null);
    }
  }
}
