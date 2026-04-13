
package org.globalart.astrum.repository.apt.internal.snapshot;

import org.globalart.astrum.repository.view.Content;

import static org.globalart.astrum.repository.apt.internal.AptMimeTypes.BZIP;
import static org.globalart.astrum.repository.apt.internal.AptMimeTypes.GZIP;
import static org.globalart.astrum.repository.apt.internal.AptMimeTypes.SIGNATURE;
import static org.globalart.astrum.repository.apt.internal.AptMimeTypes.TEXT;
import static org.globalart.astrum.repository.apt.internal.AptMimeTypes.XZ;

/**
 * @since 3.17
 */
public class SnapshotItem
{
  public static enum Role
  {
    RELEASE_INDEX(TEXT),
    RELEASE_INLINE_INDEX(TEXT),
    PACKAGE_INDEX_RAW(TEXT),
    RELEASE_SIG(SIGNATURE),
    PACKAGE_INDEX_GZ(GZIP),
    PACKAGE_INDEX_BZ2(BZIP),
    PACKAGE_INDEX_XZ(XZ),
    METADATA_RAW(TEXT),
    METADATA_GZ(GZIP),
    METADATA_BZ2(BZIP),
    METADATA_XZ(XZ);

    private final String mimeType;

    Role(final String mimeType) {
      this.mimeType = mimeType;
    }

    public String getMimeType() {
      return mimeType;
    }
  }

  public static class ContentSpecifier
  {
    public final String path;

    public final Role role;

    public final String component;

    public final String architecture;

    public ContentSpecifier(final String path, final Role role) {
      this.path = path;
      this.role = role;
      this.component = null;
      this.architecture = null;
    }

    public ContentSpecifier(final String path, final Role role, final String component, final String architecture) {
      this.path = path;
      this.role = role;
      this.component = component;
      this.architecture = architecture;
    }
  }

  public final ContentSpecifier specifier;

  public final Content content;

  public SnapshotItem(final ContentSpecifier specifier, final Content content) {
    super();
    this.specifier = specifier;
    this.content = content;
  }
}
