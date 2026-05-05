
package org.globalart.astrum.repository.p2.internal.metadata;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

import javax.annotation.Nullable;
import javax.xml.stream.XMLEventReader;
import javax.xml.stream.XMLEventWriter;
import javax.xml.stream.XMLInputFactory;
import javax.xml.stream.XMLOutputFactory;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.events.EndElement;
import javax.xml.stream.events.StartElement;
import javax.xml.stream.events.XMLEvent;

import org.globalart.astrum.common.io.SafeXml;
import org.globalart.astrum.repository.p2.internal.proxy.StreamCopier.StreamTransformer;

import org.apache.commons.io.IOUtils;

import static javax.xml.stream.XMLStreamConstants.END_ELEMENT;
import static javax.xml.stream.XMLStreamConstants.START_ELEMENT;

public abstract class XmlTransformer
    implements StreamTransformer
{
  @Override
  public void transform(final InputStream in, final OutputStream out) throws IOException {
    XMLInputFactory inputFactory = SafeXml.newXmlInputFactory();
    XMLOutputFactory outputFactory = XMLOutputFactory.newFactory();
    XMLEventReader reader = null;
    XMLEventWriter writer = null;

    try {
      reader = inputFactory.createXMLEventReader(in);
      writer = outputFactory.createXMLEventWriter(out);
      transform(reader, writer);
      writer.flush();
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
    finally {
      if (reader != null) {
        close(reader);
      }
      if (writer != null) {
        close(writer);
      }
      IOUtils.close(in);
      IOUtils.close(out);
    }
  }

  private void close(final XMLEventReader reader) {
    try {
      reader.close();
    }
    catch (Exception e) {
    }
  }

  private void close(final XMLEventWriter writer) {
    try {
      writer.close();
    }
    catch (Exception e) {
    }
  }

  protected abstract void transform(XMLEventReader reader, XMLEventWriter writer) throws XMLStreamException;

  static boolean isStartTagWithName(@Nullable final XMLEvent tag, final String name) {
    if (tag != null && tag.getEventType() == START_ELEMENT) {
      StartElement startElement = tag.asStartElement();
      if (startElement.getName().getLocalPart().equals(name)) {
        return true;
      }
    }
    return false;
  }

  static boolean isEndTagWithName(@Nullable final XMLEvent tag, final String name) {
    if (tag != null && tag.getEventType() == END_ELEMENT) {
      EndElement endElement = tag.asEndElement();
      if (endElement.getName().getLocalPart().equals(name)) {
        return true;
      }
    }
    return false;
  }
}
