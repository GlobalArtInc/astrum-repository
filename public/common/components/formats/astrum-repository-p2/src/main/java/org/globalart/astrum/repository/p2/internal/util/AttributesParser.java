
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;

import org.globalart.astrum.repository.p2.internal.exception.AttributeParsingException;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.payloads.TempBlob;

public interface AttributesParser
{
  P2Attributes getAttributesFromBlob(final TempBlob tempBlob, final String extension)
      throws IOException, AttributeParsingException;
}
