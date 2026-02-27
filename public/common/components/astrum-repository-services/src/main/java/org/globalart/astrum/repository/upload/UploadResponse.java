
package org.globalart.astrum.repository.upload;

import java.util.Collection;
import java.util.Collections;
import java.util.List;

import org.globalart.astrum.repository.view.Content;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * The asset paths that were uploaded.
 *
 * @since 3.10
 */
public class UploadResponse
{
  private final List<String> assetPaths;

  private final Collection<Content> contents;

  public UploadResponse(final List<String> assetPaths) {
    this.contents = Collections.emptyList();
    this.assetPaths = checkNotNull(assetPaths);
  }

  public UploadResponse(final Collection<Content> contents, final List<String> assetPaths) {
    this.contents = checkNotNull(contents);
    this.assetPaths = checkNotNull(assetPaths);
  }

  public UploadResponse(final Content content, final List<String> assetPaths) {
    this.contents = Collections.singletonList(checkNotNull(content));
    this.assetPaths = checkNotNull(assetPaths);
  }

  public List<String> getAssetPaths() {
    return assetPaths;
  }

  public Collection<Content> getContents() {
    return contents;
  }
}
