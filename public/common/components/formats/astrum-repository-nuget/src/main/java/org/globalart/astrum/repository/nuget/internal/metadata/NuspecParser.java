package org.globalart.astrum.repository.nuget.internal.metadata;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.regex.Pattern;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;

import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.Dependency;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.DependencyGroup;

import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.SAXException;

public final class NuspecParser
{
  private static final Pattern ID_PATTERN = Pattern.compile("^\\w+(?:[_.-]\\w+)*$");

  private static final int MAX_ID_LENGTH = 100;

  private static final long MAX_NUSPEC_SIZE = 4L * 1024 * 1024;

  private static final String[][] TEXT_ELEMENTS = {
      {"title", "title"}, {"authors", "authors"}, {"owners", "owners"}, {"description", "description"},
      {"summary", "summary"}, {"releaseNotes", "release_notes"}, {"copyright", "copyright"},
      {"language", "language"}, {"tags", "tags"}, {"projectUrl", "project_url"}, {"iconUrl", "icon_url"},
      {"licenseUrl", "license_url"}
  };

  private NuspecParser() {
  }

  public static Optional<byte[]> readNuspec(final InputStream nupkg) throws IOException {
    try (ZipInputStream zip = new ZipInputStream(nupkg)) {
      ZipEntry entry;
      while ((entry = zip.getNextEntry()) != null) {
        String name = entry.getName();
        if (!entry.isDirectory() && !name.contains("/") && name.toLowerCase(Locale.ROOT).endsWith(".nuspec")) {
          byte[] content = zip.readNBytes((int) MAX_NUSPEC_SIZE + 1);
          if (content.length > MAX_NUSPEC_SIZE) {
            throw new IOException("The .nuspec file is too large");
          }
          return Optional.of(content);
        }
      }
    }
    return Optional.empty();
  }

  public static NugetPackageMetadata parsePackage(final InputStream nupkg) throws IOException {
    byte[] nuspec = readNuspec(nupkg).orElseThrow(() -> new InvalidNugetPackageException(
        "The package does not contain a .nuspec file"));
    return parseNuspec(nuspec);
  }

  public static NugetPackageMetadata parseNuspec(final byte[] nuspec) {
    Element root = parseXml(nuspec).getDocumentElement();
    if (root == null || !"package".equals(localName(root))) {
      throw new InvalidNugetPackageException("The .nuspec file has no package element");
    }
    Element metadataElement = child(root, "metadata")
        .orElseThrow(() -> new InvalidNugetPackageException("The .nuspec file has no metadata element"));

    NugetPackageMetadata metadata = new NugetPackageMetadata();
    String id = childText(metadataElement, "id").trim();
    if (id.isEmpty() || id.length() > MAX_ID_LENGTH || !ID_PATTERN.matcher(id).matches()) {
      throw new InvalidNugetPackageException("Invalid package id '" + id + "'");
    }
    metadata.setId(id);
    String versionText = childText(metadataElement, "version").trim();
    metadata.setVersion(NugetVersion.parse(versionText)
        .orElseThrow(() -> new InvalidNugetPackageException("Invalid package version '" + versionText + "'")));

    for (String[] element : TEXT_ELEMENTS) {
      metadata.setText(element[1], childText(metadataElement, element[0]));
    }
    metadata.setText("min_client_version", metadataElement.getAttribute("minClientVersion"));
    child(metadataElement, "license").ifPresent(license -> {
      if ("expression".equalsIgnoreCase(license.getAttribute("type"))) {
        metadata.setText("license_expression", license.getTextContent());
      }
    });
    metadata.setRequireLicenseAcceptance(Boolean.parseBoolean(childText(metadataElement, "requireLicenseAcceptance").trim()));
    metadata.setDevelopmentDependency(Boolean.parseBoolean(childText(metadataElement, "developmentDependency").trim()));
    metadata.setDependencyGroups(dependencyGroups(metadataElement));
    return metadata;
  }

  private static List<DependencyGroup> dependencyGroups(final Element metadataElement) {
    List<DependencyGroup> groups = new ArrayList<>();
    Optional<Element> dependencies = child(metadataElement, "dependencies");
    if (dependencies.isEmpty()) {
      return groups;
    }
    List<Dependency> ungrouped = new ArrayList<>();
    for (Element element : children(dependencies.get())) {
      if ("group".equals(localName(element))) {
        List<Dependency> grouped = new ArrayList<>();
        for (Element dependency : children(element)) {
          dependency(dependency).ifPresent(grouped::add);
        }
        groups.add(new DependencyGroup(element.getAttribute("targetFramework"), grouped));
      }
      else {
        dependency(element).ifPresent(ungrouped::add);
      }
    }
    if (!ungrouped.isEmpty()) {
      groups.add(0, new DependencyGroup(null, ungrouped));
    }
    return groups;
  }

  private static Optional<Dependency> dependency(final Element element) {
    if (!"dependency".equals(localName(element))) {
      return Optional.empty();
    }
    String id = element.getAttribute("id").trim();
    if (id.isEmpty()) {
      return Optional.empty();
    }
    return Optional.of(new Dependency(id, element.getAttribute("version").trim()));
  }

  private static Document parseXml(final byte[] content) {
    try {
      DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
      factory.setNamespaceAware(true);
      factory.setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true);
      factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
      factory.setFeature("http://xml.org/sax/features/external-general-entities", false);
      factory.setFeature("http://xml.org/sax/features/external-parameter-entities", false);
      factory.setExpandEntityReferences(false);
      factory.setXIncludeAware(false);
      DocumentBuilder builder = factory.newDocumentBuilder();
      builder.setErrorHandler(null);
      return builder.parse(new ByteArrayInputStream(content));
    }
    catch (ParserConfigurationException | SAXException | IOException e) {
      throw new InvalidNugetPackageException("The .nuspec file is not valid XML: " + e.getMessage());
    }
  }

  private static String localName(final Node node) {
    String name = node.getLocalName();
    if (name == null) {
      name = node.getNodeName();
      int colon = name.indexOf(':');
      if (colon >= 0) {
        name = name.substring(colon + 1);
      }
    }
    return name;
  }

  private static List<Element> children(final Element parent) {
    List<Element> result = new ArrayList<>();
    NodeList nodes = parent.getChildNodes();
    for (int i = 0; i < nodes.getLength(); i++) {
      if (nodes.item(i) instanceof Element element) {
        result.add(element);
      }
    }
    return result;
  }

  private static Optional<Element> child(final Element parent, final String name) {
    return children(parent).stream().filter(element -> name.equals(localName(element))).findFirst();
  }

  private static String childText(final Element parent, final String name) {
    return child(parent, name).map(Element::getTextContent).orElse("");
  }
}
