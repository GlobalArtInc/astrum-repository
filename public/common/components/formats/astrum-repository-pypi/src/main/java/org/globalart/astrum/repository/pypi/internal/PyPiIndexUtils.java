
package org.globalart.astrum.repository.pypi.internal;

import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.function.Function;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.stream.Collectors.toList;
import static org.apache.commons.text.StringEscapeUtils.unescapeHtml4;

public final class PyPiIndexUtils
{
  private static final String PYPI_REQUIRES_PYTHON = "data-requires-python";

  private static final String PYPI_YANKED = "data-yanked";

  public static final String RELATIVE_PREFIX = "../../";

  private static final Pattern ANCHOR =
      Pattern.compile("<a\\b((?:[^>\"']|\"[^\"]*\"|'[^']*')*)>(.*?)</a\\s*>", Pattern.CASE_INSENSITIVE | Pattern.DOTALL);

  private static final Pattern ATTRIBUTE =
      Pattern.compile("([^\\s\"'>/=]+)(?:\\s*=\\s*(?:\"([^\"]*)\"|'([^']*)'|([^\\s\"'=<>`]+)))?");

  private static final Pattern TAG = Pattern.compile("<[^>]*>");

  public static List<PyPiLink> extractLinksFromIndex(final InputStream in) throws IOException {
    checkNotNull(in);
    return extractLinksFromIndex(new String(in.readAllBytes(), UTF_8));
  }

  public static List<PyPiLink> extractLinksFromIndex(final String html) {
    checkNotNull(html);
    List<PyPiLink> results = new ArrayList<>();
    Matcher anchor = ANCHOR.matcher(html);
    while (anchor.find()) {
      Map<String, String> attributes = parseAttributes(anchor.group(1));
      String href = attributes.get("href");
      if (href == null) {
        continue;
      }
      String file = unescapeHtml4(TAG.matcher(anchor.group(2)).replaceAll("")).trim();
      results.add(new PyPiLink(file, href, attributes.get(PYPI_REQUIRES_PYTHON), attributes.get(PYPI_YANKED)));
    }
    return results;
  }

  private static Map<String, String> parseAttributes(final String text) {
    Map<String, String> attributes = new HashMap<>();
    Matcher matcher = ATTRIBUTE.matcher(text);
    while (matcher.find()) {
      String name = matcher.group(1).toLowerCase(Locale.ENGLISH);
      String value = "";
      for (int group = 2; group <= 4; group++) {
        if (matcher.group(group) != null) {
          value = matcher.group(group);
          break;
        }
      }
      attributes.putIfAbsent(name, unescapeHtml4(value));
    }
    return attributes;
  }

  public static String buildIndexPage(final TemplateHelper helper, final String name, final Collection<PyPiLink> links) {
    checkNotNull(helper);
    checkNotNull(name);
    checkNotNull(links);
    URL template = PyPiIndexUtils.class.getResource("pypi-index.vm");
    TemplateParameters params = helper.parameters();
    params.set("name", name);
    params.set("assets", links.stream().map(PyPiIndexUtils::indexLinkToMap).collect(toList()));
    return helper.render(template, params);
  }

  public static String buildRootIndexPage(final TemplateHelper helper, final Collection<PyPiLink> links) {
    checkNotNull(helper);
    checkNotNull(links);
    URL template = PyPiIndexUtils.class.getResource("pypi-root-index.vm");
    TemplateParameters params = helper.parameters();
    params.set("assets", links.stream().map(PyPiIndexUtils::rootIndexLinkToMap).collect(toList()));
    return helper.render(template, params);
  }

  static Map<String, String> indexLinkToMap(final PyPiLink link) {
    Map<String, String> map = new LinkedHashMap<>();
    map.put("link", link.getLink());
    map.put("file", link.getFile());
    map.put(PYPI_REQUIRES_PYTHON, link.getDataRequiresPython());
    if (link.getYanked() != null) {
      map.put(PYPI_YANKED, link.getYanked());
    }
    return map;
  }

  static Map<String, String> rootIndexLinkToMap(final PyPiLink link) {
    Map<String, String> map = new LinkedHashMap<>();
    map.put("link", link.getLink());
    map.put("name", link.getFile());
    return map;
  }

  private static List<PyPiLink> makeLinksRelative(
      final List<PyPiLink> oldLinks,
      final Function<PyPiLink, String> linkTranslator)
  {
    checkNotNull(oldLinks);
    List<PyPiLink> newLinks = new ArrayList<>();
    for (PyPiLink oldLink : oldLinks) {
      String newLink = linkTranslator.apply(oldLink);
      if (newLink != null) {
        newLinks.add(oldLink.withLink(newLink));
      }
    }
    return newLinks;
  }

  public static boolean validateIndexLinks(final String packageName, final List<PyPiLink> links) {
    String normalizedPackageName = PyPiPathUtils.normalizeName(packageName);
    return links.stream()
        .map(PyPiLink::getFile)
        .filter(Objects::nonNull)
        .map(PyPiPathUtils::normalizeName)
        .anyMatch(file -> file.startsWith(normalizedPackageName));
  }

  public static List<PyPiLink> makeIndexLinksAstrumPaths(final String name, final InputStream in) throws IOException {
    return makePackageLinksAstrumPaths(name, extractLinksFromIndex(in));
  }

  static List<PyPiLink> makePackageLinksAstrumPaths(final String name, final List<PyPiLink> oldLinks) {
    return makeLinksRelative(oldLinks, link -> {
      String filename = link.getFile();
      String version = PyPiFileUtils.extractVersionFromFilename(filename);
      String result = RELATIVE_PREFIX + PyPiPathUtils.packagesPath(name, version, filename);
      int hashIndex = link.getLink().indexOf('#');
      if (hashIndex != -1) {
        result += link.getLink().substring(hashIndex);
      }
      return result;
    });
  }

  public static List<PyPiLink> makeRootIndexRelative(final InputStream in) throws IOException {
    return makeRootIndexLinksRelative(extractLinksFromIndex(in));
  }

  static List<PyPiLink> makeRootIndexLinksRelative(final List<PyPiLink> oldLinks) {
    return makeLinksRelative(oldLinks, PyPiIndexUtils::maybeRewriteRootLink);
  }

  private static String maybeRewriteRootLink(final PyPiLink pypiLink) {
    String link = pypiLink.getLink();
    int index = link.indexOf("/simple/");
    if (index != -1) {
      return link.substring(index + "/simple/".length());
    }
    return link;
  }

  private PyPiIndexUtils() {
  }
}
