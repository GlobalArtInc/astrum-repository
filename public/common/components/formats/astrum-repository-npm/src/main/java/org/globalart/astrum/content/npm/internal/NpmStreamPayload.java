
package org.globalart.astrum.content.npm.internal;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.List;
import java.util.Objects;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.npm.internal.NpmFieldMatcher;
import org.globalart.astrum.repository.npm.internal.NpmStreamingObjectMapper;
import org.globalart.astrum.repository.view.payloads.StreamPayload;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.fasterxml.jackson.core.JsonGenerator.Feature.AUTO_CLOSE_TARGET;
import static com.fasterxml.jackson.databind.SerializationFeature.FLUSH_AFTER_WRITE_VALUE;
import static java.util.Collections.emptyList;
import static java.util.Objects.nonNull;
import static org.globalart.astrum.content.npm.internal.NpmUtils.errorInputStream;

/**
 * NPM Specific {@link StreamPayload} that implements its own {@link #copy(InputStream, OutputStream)} method
 * to allow for streaming out the {@link #openInputStream()} directly to a given {@link OutputStream}.
 *
 * @since 3.16
 */
public class NpmStreamPayload
    extends StreamPayload
{
  private static final Logger log = LoggerFactory.getLogger(NpmStreamPayload.class);

  private String revId;

  private String packageId;

  private List<NpmFieldMatcher> fieldMatchers;

  private InputStreamFunction<MissingAssetBlobException> missingBlobInputStreamSupplier;

  public NpmStreamPayload(final InputStreamSupplier supplier, final String contentType)
  {
    super(supplier, UNKNOWN_SIZE, contentType);
  }

  public NpmStreamPayload packageId(final String packageId) {
    this.packageId = packageId;
    return this;
  }

  public NpmStreamPayload revId(final String revId) {
    this.revId = revId;
    return this;
  }

  public NpmStreamPayload fieldMatchers(final List<NpmFieldMatcher> fieldMatchers) {
    this.fieldMatchers = fieldMatchers;
    return this;
  }

  public NpmStreamPayload missingBlobInputStreamSupplier(
      final InputStreamFunction<MissingAssetBlobException> missingBlobInputStreamSupplier)
  {
    this.missingBlobInputStreamSupplier = missingBlobInputStreamSupplier;
    return this;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    try {
      return super.openInputStream();
    }
    catch (MissingAssetBlobException e) { // NOSONAR
      if (nonNull(missingBlobInputStreamSupplier)) {
        return missingBlobInputStreamSupplier.apply(e);
      }

      // ASTRUM-20040 - add logging detailing the missing handler and missing blob.
      log.warn("Missing blob for asset {} and no handler set to recover, for package '{}' and rev '{}'",
          e.getAsset(), Objects.toString(packageId, "unknown"), Objects.toString(revId, "unknown"),
          log.isDebugEnabled() ? e : null);

      return errorInputStream("Missing blob and no handler set to recover.");
    }
  }

  @Override
  public void copy(final InputStream input, final OutputStream output) throws IOException {
    new NpmStreamingObjectMapper(packageId, revId, nonNull(fieldMatchers) ? fieldMatchers : emptyList())
        .configure(FLUSH_AFTER_WRITE_VALUE, false) // we don't want the object mapper to flush all the time
        .disable(AUTO_CLOSE_TARGET)
        .readAndWrite(input, new BufferedOutputStream(output));
  }
}
