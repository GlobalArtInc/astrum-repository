
package org.globalart.astrum.repository.p2.internal.metadata;

import java.net.URI;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import javax.xml.namespace.QName;
import javax.xml.stream.XMLEventFactory;
import javax.xml.stream.XMLEventReader;
import javax.xml.stream.XMLEventWriter;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.events.Attribute;
import javax.xml.stream.events.StartElement;
import javax.xml.stream.events.XMLEvent;

public class CompositeRepositoryRewriter
    extends XmlTransformer
{
  private final URI baseUri;

  private final boolean isRoot;

  private final List<String> urls = new ArrayList<>();

  public CompositeRepositoryRewriter(final URI baseUri, final boolean isRoot) {
    this.baseUri = baseUri;
    this.isRoot = isRoot;
  }

  public List<String> getUrls() {
    return urls;
  }

  @Override
  protected void transform(final XMLEventReader reader, final XMLEventWriter writer) throws XMLStreamException {
    List<XMLEvent> buffer = new ArrayList<>();

    while (reader.hasNext()) {
      XMLEvent event = reader.nextEvent();

      if (isStartTagWithName(event, "child")) {
        buffer.add(changeLocationAttribute((StartElement) event));
      }
      else {
        buffer.add(event);
      }

      for (XMLEvent xmlEvent : buffer) {
        writer.add(xmlEvent);
      }
      buffer.clear();
    }
  }

  private XMLEvent changeLocationAttribute(final StartElement locationElement) {
    QName locationAttrName = new QName("location");
    Attribute locationAttribute = locationElement.getAttributeByName(locationAttrName);

    if (locationAttribute == null) {
      return locationElement;
    }

    String remoteUrl = baseUri.resolve(locationAttribute.getValue().trim()).toString();
    if (!remoteUrl.endsWith("/")) {
      remoteUrl += '/';
    }
    urls.add(remoteUrl);
    String path = derivePath(remoteUrl);

    return XMLEventFactory.newInstance().createStartElement(
        new QName("child"), Collections
            .singletonList(XMLEventFactory.newInstance().createAttribute(locationAttrName, path))
            .iterator(),
        Collections.emptyIterator());
  }

  private String derivePath(final String remoteUrl) {
    StringBuilder path = new StringBuilder();
    if (!isRoot) {
      path.append("../");
    }
    path.append(UriToSiteHashUtil.map(remoteUrl));
    return path.append('/').toString();
  }
}
