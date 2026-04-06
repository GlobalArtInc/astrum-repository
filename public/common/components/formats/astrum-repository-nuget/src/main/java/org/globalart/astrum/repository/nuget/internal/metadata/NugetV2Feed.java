package org.globalart.astrum.repository.nuget.internal.metadata;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.util.Collection;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.StringJoiner;

import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.Dependency;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.DependencyGroup;

public final class NugetV2Feed
{
  public static final String ATOM = "application/atom+xml;type=feed;charset=utf-8";

  public static final String ATOM_ENTRY = "application/atom+xml;type=entry;charset=utf-8";

  public static final String XML = "application/xml;charset=utf-8";

  private static final DateTimeFormatter EDM_DATE = DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSS");

  private static final String NAMESPACES = " xmlns=\"http://www.w3.org/2005/Atom\""
      + " xmlns:d=\"http://schemas.microsoft.com/ado/2007/08/dataservices\""
      + " xmlns:m=\"http://schemas.microsoft.com/ado/2007/08/dataservices/metadata\"";

  private static final String UNLISTED = "1900-01-01T00:00:00.000";

  private NugetV2Feed() {
  }

  public static String serviceDocument(final String repositoryUrl) {
    return "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n"
        + "<service xml:base=\"" + escape(repositoryUrl + "/") + "\""
        + " xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:app=\"http://www.w3.org/2007/app\""
        + " xmlns=\"http://www.w3.org/2007/app\">"
        + "<workspace><atom:title>Default</atom:title>"
        + "<collection href=\"Packages\"><atom:title>Packages</atom:title></collection>"
        + "</workspace></service>";
  }

  public static String metadataDocument() {
    StringBuilder builder = new StringBuilder();
    builder.append("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>")
        .append("<edmx:Edmx Version=\"1.0\" xmlns:edmx=\"http://schemas.microsoft.com/ado/2007/06/edmx\">")
        .append("<edmx:DataServices xmlns:m=\"http://schemas.microsoft.com/ado/2007/08/dataservices/metadata\"")
        .append(" m:DataServiceVersion=\"2.0\">")
        .append("<Schema Namespace=\"NuGetGallery.OData\" xmlns=\"http://schemas.microsoft.com/ado/2006/04/edm\">")
        .append("<EntityType Name=\"V2FeedPackage\" m:HasStream=\"true\"><Key><PropertyRef Name=\"Id\" />")
        .append("<PropertyRef Name=\"Version\" /></Key>");
    for (String property : new String[]{"Id", "Version", "NormalizedVersion", "Authors", "Copyright",
        "Dependencies", "Description", "IconUrl", "Language", "PackageHash", "PackageHashAlgorithm", "ProjectUrl",
        "ReleaseNotes", "Summary", "Tags", "Title", "MinClientVersion", "LicenseUrl"}) {
      builder.append("<Property Name=\"").append(property).append("\" Type=\"Edm.String\" Nullable=\"")
          .append("Id".equals(property) || "Version".equals(property) ? "false" : "true").append("\" />");
    }
    for (String property : new String[]{"Created", "LastUpdated", "Published"}) {
      builder.append("<Property Name=\"").append(property).append("\" Type=\"Edm.DateTime\" Nullable=\"false\" />");
    }
    for (String property : new String[]{"IsLatestVersion", "IsAbsoluteLatestVersion", "IsPrerelease",
        "RequireLicenseAcceptance", "Listed"}) {
      builder.append("<Property Name=\"").append(property).append("\" Type=\"Edm.Boolean\" Nullable=\"false\" />");
    }
    builder.append("<Property Name=\"DownloadCount\" Type=\"Edm.Int32\" Nullable=\"false\" />")
        .append("<Property Name=\"VersionDownloadCount\" Type=\"Edm.Int32\" Nullable=\"false\" />")
        .append("<Property Name=\"PackageSize\" Type=\"Edm.Int64\" Nullable=\"false\" />")
        .append("</EntityType>")
        .append("<EntityContainer Name=\"FeedContext_x0060_1\" m:IsDefaultEntityContainer=\"true\">")
        .append("<EntitySet Name=\"Packages\" EntityType=\"NuGetGallery.OData.V2FeedPackage\" />")
        .append("<FunctionImport Name=\"Search\" ReturnType=\"Collection(NuGetGallery.OData.V2FeedPackage)\"")
        .append(" EntitySet=\"Packages\" m:HttpMethod=\"GET\">")
        .append("<Parameter Name=\"searchTerm\" Type=\"Edm.String\" Mode=\"In\" />")
        .append("<Parameter Name=\"targetFramework\" Type=\"Edm.String\" Mode=\"In\" />")
        .append("<Parameter Name=\"includePrerelease\" Type=\"Edm.Boolean\" Mode=\"In\" />")
        .append("</FunctionImport>")
        .append("<FunctionImport Name=\"FindPackagesById\" ReturnType=\"Collection(NuGetGallery.OData.V2FeedPackage)\"")
        .append(" EntitySet=\"Packages\" m:HttpMethod=\"GET\">")
        .append("<Parameter Name=\"id\" Type=\"Edm.String\" Mode=\"In\" />")
        .append("</FunctionImport>")
        .append("</EntityContainer></Schema></edmx:DataServices></edmx:Edmx>");
    return builder.toString();
  }

  public static String feed(
      final String repositoryUrl,
      final String title,
      final String selfHref,
      final Collection<NugetPackageMetadata> entries,
      final Collection<NugetPackageMetadata> allVersions)
  {
    Latest latest = Latest.of(allVersions);
    StringBuilder builder = new StringBuilder();
    builder.append("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n")
        .append("<feed xml:base=\"").append(escape(repositoryUrl + "/")).append("\"").append(NAMESPACES).append(">")
        .append("<m:count>").append(entries.size()).append("</m:count>")
        .append("<id>").append(escape(repositoryUrl + "/" + selfHref)).append("</id>")
        .append("<title type=\"text\">").append(escape(title)).append("</title>")
        .append("<updated>").append(now()).append("Z</updated>")
        .append("<link rel=\"self\" title=\"").append(escape(title)).append("\" href=\"").append(escape(title))
        .append("\" />");
    for (NugetPackageMetadata metadata : entries) {
      appendEntry(builder, repositoryUrl, metadata, latest, false);
    }
    builder.append("</feed>");
    return builder.toString();
  }

  public static String entry(
      final String repositoryUrl,
      final NugetPackageMetadata metadata,
      final Collection<NugetPackageMetadata> allVersions)
  {
    StringBuilder builder = new StringBuilder("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    appendEntry(builder, repositoryUrl, metadata, Latest.of(allVersions), true);
    return builder.toString();
  }

  public static String downloadUrl(final String repositoryUrl, final NugetPackageMetadata metadata) {
    return repositoryUrl + "/package/" + metadata.getId() + "/" + metadata.getVersion().toNormalizedString();
  }

  private static void appendEntry(
      final StringBuilder builder,
      final String repositoryUrl,
      final NugetPackageMetadata metadata,
      final Latest latest,
      final boolean root)
  {
    String id = metadata.getId();
    String version = metadata.getVersion().toNormalizedString();
    String key = "Packages(Id='" + id + "',Version='" + version + "')";
    String published = edmDate(metadata.text("published"));
    builder.append("<entry").append(root ? " xml:base=\"" + escape(repositoryUrl + "/") + "\"" + NAMESPACES : "")
        .append(">")
        .append("<id>").append(escape(repositoryUrl + "/" + key)).append("</id>")
        .append("<category term=\"NuGetGallery.OData.V2FeedPackage\"")
        .append(" scheme=\"http://schemas.microsoft.com/ado/2007/08/dataservices/scheme\" />")
        .append("<link rel=\"edit\" title=\"V2FeedPackage\" href=\"").append(escape(key)).append("\" />")
        .append("<link rel=\"self\" title=\"V2FeedPackage\" href=\"").append(escape(key)).append("\" />")
        .append("<title type=\"text\">").append(escape(id)).append("</title>")
        .append("<summary type=\"text\">").append(escape(metadata.text("summary"))).append("</summary>")
        .append("<updated>").append(published).append("Z</updated>")
        .append("<author><name>").append(escape(metadata.text("authors"))).append("</name></author>")
        .append("<content type=\"application/zip\" src=\"").append(escape(downloadUrl(repositoryUrl, metadata)))
        .append("\" />")
        .append("<m:properties>");
    property(builder, "Id", id);
    property(builder, "Version", metadata.getVersion().toFullString());
    property(builder, "NormalizedVersion", version);
    property(builder, "Authors", metadata.text("authors"));
    property(builder, "Copyright", metadata.text("copyright"));
    typed(builder, "Created", "Edm.DateTime", published);
    property(builder, "Dependencies", dependencies(metadata));
    property(builder, "Description", metadata.text("description"));
    typed(builder, "DownloadCount", "Edm.Int32", "0");
    property(builder, "IconUrl", metadata.text("icon_url"));
    typed(builder, "IsLatestVersion", "Edm.Boolean", String.valueOf(latest.isLatest(metadata)));
    typed(builder, "IsAbsoluteLatestVersion", "Edm.Boolean", String.valueOf(latest.isAbsoluteLatest(metadata)));
    typed(builder, "IsPrerelease", "Edm.Boolean", String.valueOf(metadata.getVersion().isPrerelease()));
    property(builder, "Language", metadata.text("language"));
    typed(builder, "LastUpdated", "Edm.DateTime", published);
    typed(builder, "Published", "Edm.DateTime", metadata.isListed() ? published : UNLISTED);
    property(builder, "PackageHash", metadata.text("package_hash"));
    property(builder, "PackageHashAlgorithm", metadata.text("package_hash_algorithm"));
    typed(builder, "PackageSize", "Edm.Int64", String.valueOf(metadata.getPackageSize()));
    property(builder, "ProjectUrl", metadata.text("project_url"));
    property(builder, "ReleaseNotes", metadata.text("release_notes"));
    typed(builder, "RequireLicenseAcceptance", "Edm.Boolean", String.valueOf(metadata.isRequireLicenseAcceptance()));
    property(builder, "Summary", metadata.text("summary"));
    property(builder, "Tags", metadata.text("tags"));
    property(builder, "Title", metadata.text("title"));
    typed(builder, "VersionDownloadCount", "Edm.Int32", "0");
    property(builder, "MinClientVersion", metadata.text("min_client_version"));
    property(builder, "LicenseUrl", metadata.text("license_url"));
    typed(builder, "Listed", "Edm.Boolean", String.valueOf(metadata.isListed()));
    builder.append("</m:properties></entry>");
  }

  public static String dependencies(final NugetPackageMetadata metadata) {
    StringJoiner joiner = new StringJoiner("|");
    for (DependencyGroup group : metadata.getDependencyGroups()) {
      String framework = group.getTargetFramework() == null ? "" : group.getTargetFramework();
      if (group.getDependencies().isEmpty()) {
        if (!framework.isEmpty()) {
          joiner.add("::" + framework);
        }
        continue;
      }
      for (Dependency dependency : group.getDependencies()) {
        joiner.add(dependency.getId() + ":" + dependency.getRange() + ":" + framework);
      }
    }
    return joiner.toString();
  }

  private static void property(final StringBuilder builder, final String name, final String value) {
    if (value == null || value.isEmpty()) {
      builder.append("<d:").append(name).append(" m:null=\"true\" />");
    }
    else {
      builder.append("<d:").append(name).append(">").append(escape(value)).append("</d:").append(name).append(">");
    }
  }

  private static void typed(final StringBuilder builder, final String name, final String type, final String value) {
    builder.append("<d:").append(name).append(" m:type=\"").append(type).append("\">").append(escape(value))
        .append("</d:").append(name).append(">");
  }

  private static String now() {
    return OffsetDateTime.now(ZoneOffset.UTC).format(EDM_DATE);
  }

  private static String edmDate(final String isoDate) {
    if (isoDate == null || isoDate.isEmpty()) {
      return now();
    }
    try {
      return OffsetDateTime.parse(isoDate).withOffsetSameInstant(ZoneOffset.UTC).format(EDM_DATE);
    }
    catch (DateTimeParseException e) {
      return now();
    }
  }

  public static String escape(final String value) {
    if (value == null) {
      return "";
    }
    StringBuilder builder = new StringBuilder(value.length());
    for (int i = 0; i < value.length(); i++) {
      char c = value.charAt(i);
      switch (c) {
        case '&' -> builder.append("&amp;");
        case '<' -> builder.append("&lt;");
        case '>' -> builder.append("&gt;");
        case '"' -> builder.append("&quot;");
        case '\'' -> builder.append("&apos;");
        default -> {
          if (c >= 0x20 || c == '\t' || c == '\n' || c == '\r') {
            builder.append(c);
          }
        }
      }
    }
    return builder.toString();
  }

  private static final class Latest
  {
    private final Map<String, NugetVersion> latest = new HashMap<>();

    private final Map<String, NugetVersion> absoluteLatest = new HashMap<>();

    static Latest of(final Collection<NugetPackageMetadata> packages) {
      Latest result = new Latest();
      List<NugetPackageMetadata> listed = packages.stream().filter(NugetPackageMetadata::isListed)
          .sorted(Comparator.comparing(NugetPackageMetadata::getVersion)).toList();
      for (NugetPackageMetadata metadata : listed) {
        result.absoluteLatest.put(metadata.getLowerId(), metadata.getVersion());
        if (!metadata.getVersion().isPrerelease()) {
          result.latest.put(metadata.getLowerId(), metadata.getVersion());
        }
      }
      return result;
    }

    boolean isLatest(final NugetPackageMetadata metadata) {
      return metadata.getVersion().equals(latest.get(metadata.getLowerId()));
    }

    boolean isAbsoluteLatest(final NugetPackageMetadata metadata) {
      return metadata.getVersion().equals(absoluteLatest.get(metadata.getLowerId()));
    }
  }
}
