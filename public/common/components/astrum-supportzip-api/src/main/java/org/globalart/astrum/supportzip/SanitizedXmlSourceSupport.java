
package org.globalart.astrum.supportzip;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.StringReader;

import javax.xml.XMLConstants;
import javax.xml.parsers.SAXParser;
import javax.xml.parsers.SAXParserFactory;
import javax.xml.transform.Transformer;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.sax.SAXSource;
import javax.xml.transform.stream.StreamResult;
import javax.xml.transform.stream.StreamSource;

import org.globalart.astrum.common.io.SafeXml;

import org.xml.sax.InputSource;
import org.xml.sax.XMLReader;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * {@link FileContentSourceSupport} that allows XSLT rewrites of existing XML content. Note that the transform
 * is performed in memory and the results saved for the lifetime of the instance, so use on large XML files is not
 * recommended.
 *
 * @since 3.0
 */
public class SanitizedXmlSourceSupport
    extends FileContentSourceSupport
{
  private final String stylesheet;

  private byte[] content;

  /**
   * Constructor.
   */
  public SanitizedXmlSourceSupport(
      final Type type,
      final String path,
      final File file,
      final Priority priority,
      final String stylesheet)
  {
    super(type, path, file, priority);
    this.stylesheet = checkNotNull(stylesheet);
  }

  @Override
  public void prepare() throws Exception {
    super.prepare();
    checkState(content == null);
    ByteArrayOutputStream stream = new ByteArrayOutputStream();
    try (InputStream input = new BufferedInputStream(new FileInputStream(file))) {
      try (OutputStream output = new BufferedOutputStream(stream)) {

        StreamSource styleSource = new StreamSource(new StringReader(stylesheet));
        TransformerFactory transformerFactory = SafeXml.newTransformerFactory();
        Transformer transformer = transformerFactory.newTransformer(styleSource);

        SAXParserFactory parserFactory = SafeXml.newSaxParserFactory();
        parserFactory.setNamespaceAware(true);

        SAXParser parser = parserFactory.newSAXParser();
        parser.setProperty(XMLConstants.ACCESS_EXTERNAL_DTD, "");
        parser.setProperty(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");

        XMLReader reader = parser.getXMLReader();
        reader.setFeature("http://apache.org/xml/features/nonvalidating/load-external-dtd", false);

        transformer.transform(new SAXSource(reader, new InputSource(input)), new StreamResult(output));
      }
    }
    content = stream.toByteArray();
  }

  @Override
  public long getSize() {
    checkState(content != null);
    return content.length;
  }

  @Override
  public InputStream getContent() throws Exception {
    checkState(content != null);
    log.debug("Reading: {} from memory", file);
    return new BufferedInputStream(new ByteArrayInputStream(content));
  }
}
