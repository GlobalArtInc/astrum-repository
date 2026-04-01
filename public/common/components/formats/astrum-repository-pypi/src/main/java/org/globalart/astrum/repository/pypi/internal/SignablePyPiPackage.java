
package org.globalart.astrum.repository.pypi.internal;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.payloads.TempBlobPartPayload;

import static com.google.common.base.Preconditions.checkNotNull;

public class SignablePyPiPackage
{
  private final TempBlobPartPayload wheelPayload;

  private final Map<String, String> attributes;

  private final TempBlobPartPayload gpgSignature;

  public SignablePyPiPackage(
      final TempBlobPartPayload wheelPayload,
      final Map<String, String> attributes,
      @Nullable final TempBlobPartPayload gpgSignature)
  {
    this.wheelPayload = checkNotNull(wheelPayload);
    this.attributes = checkNotNull(attributes);
    this.gpgSignature = gpgSignature;
  }

  public TempBlobPartPayload getWheelPayload() {
    return wheelPayload;
  }

  public Map<String, String> getAttributes() {
    return attributes;
  }

  @Nullable
  public TempBlobPartPayload getGpgSignature() {
    return gpgSignature;
  }
}
