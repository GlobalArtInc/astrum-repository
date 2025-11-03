
package org.globalart.astrum.mime;

import java.io.IOException;
import java.io.InputStream;
import java.util.List;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

/**
 * A utility component for working with MIME type detection, either "without touching" (the content), that is
 * "best effort guess" based, or doing proper MIME type detection based on MIME magic database. All these methods
 * returns in "worst case" the "unknown" MIME type: {@code application/octet-stream}, never {@code null} or empty list.
 *
 * @since 2.0
 */
public interface MimeSupport
{
  /**
   * Makes a "guess" (usually based on file extension) about the MIME type that is most applicable to the given path
   * taking into consideration the requester MimeRulesSource MIME rules. When no "hard rule" present from
   * {@link MimeRulesSource}, this method falls back to Tika mime types defined in system. The "guess" is fast,
   * but not so precise as detection, where content is needed. This method should be used whenever a MIME type of a
   * file item that is contained <b>within</b> given context that has MimeRulesSource is about to be guessed.
   *
   * @param path to guess for.
   * @param mimeRulesSources optionally, the mimeRulesSource to apply/mixin into results.
   * @return the most applicable MIME type as String.
   */
  @Nonnull
  String guessMimeTypeFromPath(String path, MimeRulesSource... mimeRulesSources);

  /**
   * Makes a "guess" (usually based on file extension) about the MIME type that is most applicable to the given path
   * taking into consideration the requester MimeRulesSource MIME rules. When no "hard rule" present from
   * {@link MimeRulesSource}, this method falls back to Tika mime types defined in system. The "guess" is fast,
   * but not so precise as detection, where content is needed. This method should be used whenever a MIME type of a
   * file item that is contained <b>within</b> given context that has MimeRulesSource is about to be guessed. The list
   * of mime types are in descending order (most specific 1st, least specific last).
   *
   * @param path to guess for.
   * @param mimeRulesSources optionally, the mimeRulesSource to apply/mixin into results.
   * @return the list of applicable mime types.
   * @since 3.0
   */
  @Nonnull
  List<String> guessMimeTypesListFromPath(String path, MimeRulesSource... mimeRulesSources);

  /**
   * Returns the most specific MIME type from result got from {@link #detectMimeType(InputStream, String)}.
   *
   * @see #detectMimeTypes(InputStream, String)
   * @since 3.0
   */
  @Nonnull
  String detectMimeType(InputStream input, @Nullable String fileName) throws IOException;

  /**
   * Returns all the detected MIME types of given input. Providing {@code fileName} just improves detection, but is
   * not mandatory. The list of mime types are in descending order (most specific 1st, least specific last).
   *
   * @since 3.0
   */
  @Nonnull
  List<String> detectMimeTypes(InputStream input, @Nullable String fileName) throws IOException;
}
