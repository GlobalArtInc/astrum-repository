
package org.globalart.astrum.content.maven.internal.recipe;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.content.maven.MavenArchetypeCatalogFacet;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.content.maven.internal.event.RebuildMavenArchetypeCatalogEvent;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.HashType;
import org.globalart.astrum.repository.maven.internal.Constants;
import org.globalart.astrum.repository.maven.internal.utils.HashedPayload;
import org.globalart.astrum.repository.view.Payload;

import com.google.common.collect.Iterables;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import com.google.common.hash.HashCode;
import org.apache.commons.lang3.StringUtils;
import org.apache.maven.archetype.catalog.Archetype;
import org.apache.maven.archetype.catalog.ArchetypeCatalog;
import org.springframework.beans.factory.annotation.Value;

import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.maven.internal.MavenModels.writeArchetypeCatalog;
import static org.globalart.astrum.repository.maven.internal.utils.MavenIOUtils.createStreamPayload;
import static org.globalart.astrum.repository.maven.internal.utils.MavenIOUtils.hashesToPayloads;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_XML;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Rebuilds the maven archetype catalog for a given repository.
 *
 * @since 3.25
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenArchetypeCatalogFacetImpl
    extends FacetSupport
    implements MavenArchetypeCatalogFacet
{
  private static final String HOSTED_ARCHETYPE_CATALOG = "hosted-archetype-catalog";

  private static final String XML = "xml";

  public static final String MAVEN_ARCHETYPE_KIND = "maven-archetype";

  private static final String ARCHETYPE_CATALOG_PATH = "/" + Constants.ARCHETYPE_CATALOG_FILENAME;

  private MavenPath archetypeCatalogMavenPath;

  private MavenContentFacet mavenContentFacet;

  private final int componentPageSize;

  @Autowired
  public MavenArchetypeCatalogFacetImpl(
      @Value("${maven.archetypes.page.size:10}") final int componentPageSize)
  {
    this.componentPageSize = componentPageSize;
  }

  @Override
  protected void doInit(final Configuration configuration) throws Exception {
    super.doInit(configuration);
    mavenContentFacet = facet(MavenContentFacet.class);
    this.archetypeCatalogMavenPath = mavenContentFacet.getMavenPathParser()
        .parsePath(ARCHETYPE_CATALOG_PATH);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RebuildMavenArchetypeCatalogEvent event) throws IOException {
    if (StringUtils.equals(getRepository().getName(), event.getRepositoryName())) {
      deleteExistingCatalog();
    }
  }

  @Override
  public void rebuildArchetypeCatalog() throws IOException {
    deleteExistingCatalog();

    log.debug("Rebuilding hosted archetype catalog for {}", getRepository().getName());

    Path path = Files.createTempFile(HOSTED_ARCHETYPE_CATALOG, XML);
    ArchetypeCatalog hostedCatalog = createArchetypeCatalog();

    try {
      HashedPayload hashedPayload = createArchetypeCatalogFile(hostedCatalog, path);
      try (Payload payload = hashedPayload.getPayload()) {
        mavenContentFacet.put(archetypeCatalogMavenPath, payload);
        putHashedContent(archetypeCatalogMavenPath, hashedPayload);
        log.trace("Rebuilt hosted archetype catalog for {} with {} archetype",
            getRepository().getName(), hostedCatalog.getArchetypes().size());
      }
    }
    finally {
      Files.delete(path);
    }
  }

  private void deleteExistingCatalog() throws IOException {
    mavenContentFacet.delete(archetypeCatalogMavenPath);
    for (HashType hashType : HashType.values()) {
      mavenContentFacet.delete(archetypeCatalogMavenPath.main().hash(hashType));
    }
  }

  private void putHashedContent(final MavenPath mavenPath, final HashedPayload hashedPayload) throws IOException {
    Map<HashAlgorithm, HashCode> hashCodes = hashedPayload.getHashCodes();
    for (Entry<HashType, Payload> entry : hashesToPayloads(hashCodes).entrySet()) {
      mavenContentFacet.put(mavenPath.hash(entry.getKey()), entry.getValue());
    }
  }

  private ArchetypeCatalog createArchetypeCatalog() {
    ArchetypeCatalog hostedCatalog = new ArchetypeCatalog();
    Iterables.addAll(hostedCatalog.getArchetypes(), getArchetypes());
    return hostedCatalog;
  }

  private HashedPayload createArchetypeCatalogFile(
      final ArchetypeCatalog hostedCatalog,
      final Path path) throws IOException
  {
    return createStreamPayload(
        path,
        APPLICATION_XML,
        (final OutputStream outputStream) -> writeArchetypeCatalog(outputStream, hostedCatalog));
  }

  private Iterable<Archetype> getArchetypes() {
    List<Archetype> archetypes = new ArrayList<>();

    FluentQuery<FluentComponent> archetypeQuery = mavenContentFacet.components().byKind(MAVEN_ARCHETYPE_KIND);
    Continuation<FluentComponent> components = archetypeQuery.browse(componentPageSize, null);
    while (!components.isEmpty()) {
      archetypes.addAll(components.stream().map(this::toArchetype).collect(toList()));
      components = archetypeQuery.browse(componentPageSize, components.nextContinuationToken());
    }
    return archetypes;
  }

  private Archetype toArchetype(final FluentComponent component) {
    Archetype archetype = new Archetype();
    archetype.setGroupId(component.namespace());
    archetype.setArtifactId(component.name());
    archetype.setVersion(component.version());
    return archetype;
  }
}
