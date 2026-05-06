
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;
import java.util.Optional;
import java.util.PropertyResourceBundle;
import java.util.jar.JarEntry;
import java.util.jar.JarInputStream;

import javax.annotation.Nullable;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import javax.xml.xpath.XPath;
import javax.xml.xpath.XPathExpressionException;
import javax.xml.xpath.XPathFactory;

import org.globalart.astrum.common.io.SafeXml;
import org.globalart.astrum.repository.p2.internal.exception.AttributeParsingException;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes.Builder;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.w3c.dom.Document;
import org.w3c.dom.Node;
import org.xml.sax.SAXException;

import static javax.xml.xpath.XPathConstants.NODE;

@Component
public class AttributesParserFeatureXml
    implements AttributesParser
{
  private static final String XML_VERSION_PATH = "feature/@version";

  private static final String XML_ID_PATH = "feature/@id";

  private static final String XML_PLUGIN_PATH = "feature/@plugin";

  private static final String XML_LABEL_PATH = "feature/@label";

  private static final String XML_FILE_NAME = "feature.xml";

  private static final String FEATURE_PROPERTIES = "feature";

  private final JarExtractor<Document> documentJarExtractor;

  private final PropertyParser propertyParser;

  private final DocumentBuilderFactory documentBuilderFactory;

  @Autowired
  public AttributesParserFeatureXml(final TempBlobConverter tempBlobConverter, final PropertyParser propertyParser)
      throws ParserConfigurationException
  {
    this.propertyParser = propertyParser;

    documentBuilderFactory = SafeXml.newdocumentBuilderFactory();
    documentBuilderFactory.setValidating(false);

    documentJarExtractor = new JarExtractor<Document>(tempBlobConverter)
    {
      @Override
      protected Document createSpecificEntity(final JarInputStream jis, final JarEntry jarEntry)
          throws IOException, AttributeParsingException
      {
        try {
          return documentBuilderFactory.newDocumentBuilder().parse(jis);
        }
        catch (ParserConfigurationException | SAXException e) {
          throw new AttributeParsingException(e);
        }
      }
    };
  }

  @Override
  public P2Attributes getAttributesFromBlob(final TempBlob tempBlob, final String extension)
      throws IOException, AttributeParsingException
  {
    Builder p2AttributesBuilder = P2Attributes.builder();
    Optional<Document> featureXmlOpt = documentJarExtractor.getSpecificEntity(tempBlob, extension, XML_FILE_NAME);

    if (featureXmlOpt.isPresent()) {
      Document document = featureXmlOpt.get();
      Optional<PropertyResourceBundle> propertiesOpt =
          propertyParser.getBundleProperties(tempBlob, extension, FEATURE_PROPERTIES);
      String featureId = extractValueFromDocument(XML_ID_PATH, document);
      if (featureId == null) {
        featureId = extractValueFromDocument(XML_PLUGIN_PATH, document);
      }

      p2AttributesBuilder
          .componentName(propertyParser.extractValueFromProperty(featureId, propertiesOpt))
          .pluginName(
              propertyParser.extractValueFromProperty(extractValueFromDocument(XML_LABEL_PATH, document), propertiesOpt))
          .componentVersion(extractValueFromDocument(XML_VERSION_PATH, document));
    }

    return p2AttributesBuilder.build();
  }

  @Nullable
  private String extractValueFromDocument(
      final String path,
      final Document from) throws AttributeParsingException
  {
    XPath xPath = XPathFactory.newInstance().newXPath();
    Node node;
    try {
      node = (Node) xPath.evaluate(path, from, NODE);
    }
    catch (XPathExpressionException e) {
      throw new AttributeParsingException(e);
    }
    if (node != null) {
      return node.getNodeValue();
    }

    return null;
  }
}
