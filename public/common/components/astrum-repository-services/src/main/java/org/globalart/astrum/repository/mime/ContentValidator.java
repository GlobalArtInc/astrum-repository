
package org.globalart.astrum.repository.mime;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.InvalidContentException;

/**
 * Content validator interface.
 *
 * @since 3.0
 */
public interface ContentValidator
{
  /**
   * Determines or confirms the content type for the given content, or throws {@link InvalidContentException} if it
   * cannot.
   *
   * @param strictContentTypeValidation whether the check should be strict or not.
   * @param contentSupplier the supplier of the content to determine or confirm content type.
   * @param contentName blob name, usually a file path or file name or just extension
   *          (file extension is used to determine content type along with "magic" detection
   *          where actual content bits are used, like file headers or magic bytes). Is
   *          optional, but be aware that if present it improves content type detection
   *          reliability.
   * @param mimeRulesSource if non-null, mime rules source to use.
   * @param declaredContentType if non-null, the declared content type will be confirmed, if null, this method
   *          will attempt to determine the content type.
   * @return the content type of the content.
   * @throws InvalidContentException if type cannot be confirmed or detected.
   */
  @Nonnull
  String determineContentType(
      boolean strictContentTypeValidation,
      InputStreamSupplier contentSupplier,
      @Nullable MimeRulesSource mimeRulesSource,
      @Nullable String contentName,
      @Nullable String declaredContentType) throws IOException;
}
