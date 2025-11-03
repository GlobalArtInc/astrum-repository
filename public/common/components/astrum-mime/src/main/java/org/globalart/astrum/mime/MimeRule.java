
package org.globalart.astrum.mime;

import java.util.Arrays;
import java.util.List;

import javax.annotation.Nonnull;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Astrum MIME type rule.
 *
 * @since 3.0
 */
public class MimeRule
{
  private final boolean override;

  private final List<String> mimetypes;

  public MimeRule(
      final boolean override,
      final String... mimetypes)
  {
    this(override, Arrays.asList(mimetypes));
  }

  public MimeRule(
      final boolean override,
      final List<String> mimetypes)
  {
    checkNotNull(mimetypes, "mimetypes");
    checkArgument(!mimetypes.isEmpty(), "mimetypes");
    this.override = override;
    this.mimetypes = ImmutableList.copyOf(mimetypes);
  }

  /**
   * If {@code true}, only this mapping should be considered, otherwise other sources like Tika might be consulted
   * as well.
   */
  public boolean isOverride() {
    return override;
  }

  /**
   * Returns the applicable MIME types for this mapping in priority order (first most applicable). Returned list is
   * immutable.
   */
  @Nonnull
  public List<String> getMimetypes() {
    return mimetypes;
  }
}
