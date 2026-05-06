
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.repository.p2.internal.exception.AttributeParsingException;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class P2TempBlobUtils
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AttributesParserFeatureXml featureXmlParser;

  private final AttributesParserManifest manifestParser;

  @Autowired
  public P2TempBlobUtils(final AttributesParserFeatureXml featureXmlParser,
                         final AttributesParserManifest manifestParser)
  {
    this.featureXmlParser = checkNotNull(featureXmlParser);
    this.manifestParser = checkNotNull(manifestParser);
  }

  public P2Attributes mergeAttributesFromTempBlob(final TempBlob tempBlob, final P2Attributes sourceP2Attributes) {
    checkNotNull(sourceP2Attributes.getExtension());
    P2Attributes p2Attributes = null;
    try {
      p2Attributes = featureXmlParser.getAttributesFromBlob(tempBlob, sourceP2Attributes.getExtension());
      if (p2Attributes.isEmpty()) {
        p2Attributes = manifestParser.getAttributesFromBlob(tempBlob, sourceP2Attributes.getExtension());
      }
    }
    catch (AttributeParsingException | IOException | RuntimeException ex) {
      log.warn("Could not read p2 attributes of {}: {}", sourceP2Attributes.getPath(), ex.toString());
    }

    return Optional.ofNullable(p2Attributes)
        .filter(jarP2Attributes -> !jarP2Attributes.isEmpty())
        .map(jarP2Attributes -> P2Attributes.builder().merge(sourceP2Attributes, jarP2Attributes).build())
        .orElse(sourceP2Attributes);
  }
}
