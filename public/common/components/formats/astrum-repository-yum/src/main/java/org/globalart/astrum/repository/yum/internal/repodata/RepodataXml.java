package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

import javax.xml.stream.XMLInputFactory;
import javax.xml.stream.XMLOutputFactory;
import javax.xml.stream.XMLStreamConstants;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamReader;
import javax.xml.stream.XMLStreamWriter;

public final class RepodataXml
{
  public static final String COMMON_NS = "http://linux.duke.edu/metadata/common";

  public static final String RPM_NS = "http://linux.duke.edu/metadata/rpm";

  public static final String FILELISTS_NS = "http://linux.duke.edu/metadata/filelists";

  public static final String OTHER_NS = "http://linux.duke.edu/metadata/other";

  public static final String REPO_NS = "http://linux.duke.edu/metadata/repo";

  public static final String XML_NS = "http://www.w3.org/XML/1998/namespace";

  private static final XMLInputFactory INPUT_FACTORY = inputFactory();

  private static final XMLOutputFactory OUTPUT_FACTORY = outputFactory();

  private RepodataXml() {
  }

  public static XMLStreamReader reader(final InputStream in) throws XMLStreamException {
    return INPUT_FACTORY.createXMLStreamReader(in);
  }

  public static final byte[] DECLARATION = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".getBytes(StandardCharsets.UTF_8);

  public static XMLStreamWriter writer(final OutputStream out) throws XMLStreamException {
    try {
      out.write(DECLARATION);
    }
    catch (IOException e) {
      throw new XMLStreamException(e);
    }
    return OUTPUT_FACTORY.createXMLStreamWriter(out, "UTF-8");
  }

  public static String clean(final String text) {
    StringBuilder builder = null;
    for (int i = 0; i < text.length(); i++) {
      char c = text.charAt(i);
      boolean valid = c == '\t' || c == '\n' || c == '\r' || (c >= 0x20 && c <= 0xD7FF)
          || (c >= 0xE000 && c <= 0xFFFD) || Character.isSurrogate(c);
      if (!valid) {
        if (builder == null) {
          builder = new StringBuilder(text.length());
          builder.append(text, 0, i);
        }
      }
      else if (builder != null) {
        builder.append(c);
      }
    }
    return builder == null ? text : builder.toString();
  }

  public static boolean nextElement(final XMLStreamReader reader) throws XMLStreamException {
    while (reader.hasNext()) {
      if (reader.next() == XMLStreamConstants.START_ELEMENT) {
        return true;
      }
    }
    return false;
  }

  public static void skipElement(final XMLStreamReader reader) throws XMLStreamException {
    int depth = 1;
    while (depth > 0 && reader.hasNext()) {
      int event = reader.next();
      if (event == XMLStreamConstants.START_ELEMENT) {
        depth++;
      }
      else if (event == XMLStreamConstants.END_ELEMENT) {
        depth--;
      }
    }
  }

  public static void copyElement(final XMLStreamReader reader, final XMLStreamWriter writer)
      throws XMLStreamException
  {
    int depth = 0;
    do {
      switch (reader.getEventType()) {
        case XMLStreamConstants.START_ELEMENT -> {
          depth++;
          String namespace = reader.getNamespaceURI();
          String prefix = reader.getPrefix();
          if (namespace == null || namespace.isEmpty()) {
            writer.writeStartElement(reader.getLocalName());
          }
          else {
            writer.writeStartElement(prefix == null ? "" : prefix, reader.getLocalName(), namespace);
          }
          for (int i = 0; i < reader.getAttributeCount(); i++) {
            String attributeNamespace = reader.getAttributeNamespace(i);
            if (attributeNamespace == null || attributeNamespace.isEmpty()) {
              writer.writeAttribute(reader.getAttributeLocalName(i), reader.getAttributeValue(i));
            }
            else {
              String attributePrefix = reader.getAttributePrefix(i);
              writer.writeAttribute(attributePrefix == null ? "" : attributePrefix, attributeNamespace,
                  reader.getAttributeLocalName(i), reader.getAttributeValue(i));
            }
          }
        }
        case XMLStreamConstants.END_ELEMENT -> {
          depth--;
          writer.writeEndElement();
        }
        case XMLStreamConstants.CHARACTERS, XMLStreamConstants.SPACE ->
            writer.writeCharacters(reader.getTextCharacters(), reader.getTextStart(), reader.getTextLength());
        case XMLStreamConstants.CDATA -> writer.writeCharacters(reader.getText());
        default -> {
        }
      }
      if (depth > 0) {
        reader.next();
      }
    }
    while (depth > 0);
  }

  private static XMLInputFactory inputFactory() {
    XMLInputFactory factory = XMLInputFactory.newFactory();
    factory.setProperty(XMLInputFactory.SUPPORT_DTD, false);
    factory.setProperty(XMLInputFactory.IS_SUPPORTING_EXTERNAL_ENTITIES, false);
    factory.setProperty(XMLInputFactory.IS_NAMESPACE_AWARE, true);
    factory.setProperty(XMLInputFactory.IS_COALESCING, true);
    return factory;
  }

  private static XMLOutputFactory outputFactory() {
    XMLOutputFactory factory = XMLOutputFactory.newFactory();
    factory.setProperty(XMLOutputFactory.IS_REPAIRING_NAMESPACES, true);
    return factory;
  }
}
