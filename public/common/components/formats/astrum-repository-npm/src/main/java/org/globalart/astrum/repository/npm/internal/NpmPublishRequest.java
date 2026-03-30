
package org.globalart.astrum.repository.npm.internal;

import java.io.Closeable;
import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * Parsed "npm publish" or "npm unpublish" request containing the JSON contents (exclusive of attachments) and any
 * included attachments as TempBlobs. The "data" field of each attachment is replaced with a string that can be used to
 * fetch the blob.
 *
 * Note that this class should be used within a try-with-resources statement for safety, as it is intended that it will
 * manage its own temp blobs.
 *
 * @since 3.4
 */
public class NpmPublishRequest
    implements Closeable
{
  private final NestedAttributesMap packageRoot;

  private final Map<String, TempBlob> tempBlobs;

  public NpmPublishRequest(final NestedAttributesMap packageRoot, final Map<String, TempBlob> tempBlobs) {
    this.packageRoot = checkNotNull(packageRoot);
    this.tempBlobs = checkNotNull(tempBlobs);
  }

  public NestedAttributesMap getPackageRoot() {
    return packageRoot;
  }

  public TempBlob requireBlob(final String data) {
    TempBlob blob = tempBlobs.get(data);
    checkState(blob != null, "Missing temporary blob: " + data);
    return blob;
  }

  @Override
  public void close() {
    for (TempBlob tempBlob : tempBlobs.values()) {
      tempBlob.close();
    }
  }
}
