
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.PartPayload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * PartPayload backed by a TempBlob. Useful for file uploads surrounded by other multipart data.
 *
 * @since 3.1
 */
public class TempBlobPartPayload
    extends TempBlobPayload
    implements PartPayload
{
  private final String name;

  private final String fieldName;

  private final boolean isFormField;

  private final PartPayload sourcePayload;

  public TempBlobPartPayload(final PartPayload payload, final TempBlob tempBlob) throws IOException {
    super(tempBlob, checkNotNull(payload).getContentType());
    this.name = payload.getName();
    this.fieldName = payload.getFieldName();
    this.isFormField = payload.isFormField();
    this.sourcePayload = payload;
  }

  /**
   * @since 3.16
   */
  public TempBlobPartPayload(
      final String fieldName,
      final boolean isFormField,
      @Nullable final String name,
      @Nullable final String contentType,
      final TempBlob tempBlob)
  {
    super(tempBlob, contentType);
    this.fieldName = checkNotNull(fieldName);
    this.name = name;
    this.isFormField = isFormField;
    this.sourcePayload = null;
  }

  @Nullable
  @Override
  public String getName() {
    return name;
  }

  @Override
  public String getFieldName() {
    return fieldName;
  }

  @Override
  public boolean isFormField() {
    return isFormField;
  }

  @Nullable
  @Override
  public String getHeader(final String name) {
    return sourcePayload != null ? sourcePayload.getHeader(name) : null;
  }
}
