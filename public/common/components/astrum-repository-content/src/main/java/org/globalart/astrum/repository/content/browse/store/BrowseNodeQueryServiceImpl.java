
package org.globalart.astrum.repository.content.browse.store;

import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.template.EscapeHelper;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.BrowseListItem;
import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowseNodeComparator;
import org.globalart.astrum.repository.browse.node.BrowseNodeFilter;
import org.globalart.astrum.repository.browse.node.BrowseNodeIdentity;
import org.globalart.astrum.repository.browse.node.BrowseNodeQueryService;
import org.globalart.astrum.repository.browse.node.DefaultBrowseNodeComparator;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.repository.selector.ContentAuthHelper;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.selector.JexlSelector;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorFilterBuilder;
import org.globalart.astrum.selector.SelectorManager;

import com.google.common.base.Equivalence;
import com.google.common.base.Equivalence.Wrapper;
import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.asList;
import static java.util.Collections.emptyList;
import static java.util.stream.Collectors.joining;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH;
import static org.globalart.astrum.repository.browse.node.BrowsePath.SLASH_CHAR;
import static org.globalart.astrum.repository.content.facet.ContentFacetFinder.findContentFacets;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;

/**
 * New-DB implementation of {@link BrowseNodeQueryService}.
 *
 * @since 3.26
 */
@Component
public class BrowseNodeQueryServiceImpl
    extends StateGuardLifecycleSupport
    implements BrowseNodeQueryService
{
  private static final DateTimeFormatter DATE_TIME_FORMAT = DateTimeFormatter.ofPattern("EEE MMM dd HH:mm:ss zzz yyyy");

  private static final EscapeHelper escapeHelper = new EscapeHelper();

  private final SecurityHelper securityHelper;

  private final SelectorManager selectorManager;

  private final Map<String, BrowseNodeFilter> browseNodeFilters;

  private final Map<String, BrowseNodeIdentity> browseNodeIdentities;

  private final Map<String, BrowseNodeComparator> browseNodeComparators;

  private final BrowseNodeComparator defaultBrowseNodeComparator;

  private final ContentAuthHelper contentAuthHelper;

  private final SelectorFilterBuilder selectorFilterBuilder;

  @Autowired
  public BrowseNodeQueryServiceImpl(
      final SecurityHelper securityHelper,
      final SelectorManager selectorManager,
      final List<BrowseNodeFilter> browseNodeFilters,
      final List<BrowseNodeIdentity> browseNodeIdentitiesList,
      final List<BrowseNodeComparator> browseNodeComparatorsList,
      final ContentAuthHelper contentAuthHelper,
      final SelectorFilterBuilder selectorFilterBuilder)
  {
    this.securityHelper = checkNotNull(securityHelper);
    this.selectorManager = checkNotNull(selectorManager);
    this.browseNodeFilters = QualifierUtil.buildQualifierBeanMap(checkNotNull(browseNodeFilters));
    this.browseNodeIdentities = QualifierUtil.buildQualifierBeanMap(checkNotNull(browseNodeIdentitiesList));
    this.browseNodeComparators = QualifierUtil.buildQualifierBeanMap(checkNotNull(browseNodeComparatorsList));
    this.defaultBrowseNodeComparator = checkNotNull(this.browseNodeComparators.get(DefaultBrowseNodeComparator.NAME));
    this.contentAuthHelper = checkNotNull(contentAuthHelper);
    this.selectorFilterBuilder = checkNotNull(selectorFilterBuilder);
  }

  @Override
  public Iterable<BrowseNode> getByPath(
      final Repository repository,
      final List<String> displayPath,
      final int maxNodes)
  {
    String repositoryName = repository.getName();
    String format = repository.getFormat().getValue();

    // Always fetch selectors to ensure content selectors are applied as additional restrictions
    List<SelectorConfiguration> selectors;
    try {
      selectors = selectorManager.browseActive(asList(repositoryName), asList(format));
    }
    catch (Exception e) {
      log.warn("Failed to fetch content selectors for repository {} with format {}", repositoryName, format, e);
      selectors = emptyList();
    }

    final boolean canBrowseRepository = hasBrowsePermission(repositoryName, format);

    if (!canBrowseRepository && selectors.isEmpty()) {
      return emptyList(); // no browse permission and no selectors -> no results
    }

    // PRIVILEGE HIERARCHY: Only apply SQL content selector filters if user lacks browse permission
    // Full repository permissions take precedence over content selector restrictions
    final Map<String, Object> filterParameters;
    final String contentFilter;
    if (canBrowseRepository) {
      filterParameters = null;
      contentFilter = null;
    }
    else {
      filterParameters = new HashMap<>();
      contentFilter = buildContentAuthFilter(repository, selectors, filterParameters);
    }
    final boolean hasJexl = selectors.stream().anyMatch(this::isJexl);

    List<BrowseNode> results;
    if (repository.getType() instanceof GroupType) {
      int repositoryId = contentRepositoryId(repository).orElse(-1); // -1 if no content facet, like a plain group

      BrowseNodeIdentity identity = browseNodeIdentities.getOrDefault(format, BrowseNode::getName);
      Equivalence<BrowseNode> browseNodeEquivalence = Equivalence.equals().onResultOf(identity::identity);
      BrowseNodeFilter filter = browseNodeFilters.getOrDefault(format, (node, name) -> true);

      // overlay member results, first-one-wins if there are any nodes with the same name
      results = members(repository)
          .map(m -> selectByPath(m, displayPath, maxNodes, contentFilter, filterParameters, hasJexl,
              canBrowseRepository))
          .flatMap(List::stream)
          .map(browseNodeEquivalence::wrap)
          .distinct()
          .map(Wrapper::get)
          .filter(node -> filter.test(node, repositoryId == ((BrowseNodeData) node).repositoryId))
          .limit(maxNodes)
          .collect(toList());
    }
    else {
      results = selectByPath(repository, displayPath, maxNodes, contentFilter, filterParameters, hasJexl,
          canBrowseRepository);
    }

    results.sort(getBrowseNodeComparator(format));

    return results;
  }

  /**
   * Retrieve the browse nodes directly under the given display path in the selected repository.
   */
  private List<BrowseNode> selectByPath(
      final Repository repository,
      final List<String> displayPath,
      final int maxNodes,
      @Nullable final String contentFilter,
      @Nullable final Map<String, Object> filterParams,
      final boolean hasJexl,
      final boolean canBrowseRepository)
  {
    List<BrowseNode> nodes = repository.optionalFacet(BrowseFacet.class)
        .map(facet -> facet.getByDisplayPath(displayPath, maxNodes, contentFilter, filterParams))
        .orElse(ImmutableList.of());

    if (hasJexl) {
      // PRIVILEGE HIERARCHY: If user has full browse permission, skip content selector filtering
      // Full repository permissions take precedence over content selector restrictions
      if (!canBrowseRepository) {
        // additional filtering that we couldn't do in SQL
        String repositoryName = repository.getName();
        String format = repository.getFormat().getValue();
        nodes = nodes.stream()
            .filter(node -> contentAuthHelper.checkPathPermissionsJexlOnly(node.getPath(), format, repositoryName))
            .collect(toList());
      }
    }

    return nodes;
  }

  /**
   * Does the current user have permission to browse the full repository?
   */
  private boolean hasBrowsePermission(final String repositoryName, final String format) {
    return securityHelper.anyPermitted(new RepositoryViewPermission(format, repositoryName, BreadActions.BROWSE));
  }

  /**
   * Returns stream of all non-group repositories reachable from the given repository.
   */
  private static Stream<Repository> members(final Repository repository) {
    return repository.facet(GroupFacet.class).allMembers().stream();
  }

  /**
   * Returns the potentially format-specific comparator to use for sorting browse nodes.
   */
  private Comparator<BrowseNode> getBrowseNodeComparator(final String format) {
    return browseNodeComparators.getOrDefault(format, defaultBrowseNodeComparator);
  }

  private boolean isJexl(final SelectorConfiguration selectorConfiguration) {
    return JexlSelector.TYPE.equals(selectorConfiguration.getType());
  }

  /**
   * Builds a browse node filter in SQL for the current user.
   */
  @Nullable
  private String buildContentAuthFilter(
      final Repository repository,
      final List<SelectorConfiguration> selectors,
      final Map<String, Object> filterParameters)
  {
    return selectorFilterBuilder.buildFilter(repository.getFormat().getValue(), "B.request_path", selectors,
        filterParameters);
  }

  @Override
  public List<BrowseListItem> toListItems(final Repository repository, final Iterable<BrowseNode> nodes) {
    List<BrowseListItem> items = new ArrayList<>();
    if (nodes == null) {
      return items;
    }

    for (BrowseNode node : nodes) {
      String lastModified = null;
      String size = null;
      String link;

      if (node.isLeaf()) {
        Optional<FluentAsset> asset = getAssetById(repository, node.getAssetId());

        if (!asset.isPresent()) {
          log.debug("Could not find expected asset (id): {} ({}) in repository: {}",
              node.getPath(), node.getAssetId(), repository.getName());
          // expected an asset here but it's missing, move along to the next node
          continue;
        }

        Optional<AssetBlob> blob = asset.get().blob();
        if (blob.isPresent()) {
          // blobs are immutable so "last-modified" is the creation time of the latest blob
          lastModified = DATE_TIME_FORMAT.format(blob.get().blobCreated().toZonedDateTime());
          size = Long.toString(blob.get().blobSize());
        }

        link = getAssetLink(repository, asset.get());
      }
      else {
        link = escapeHelper.uri(node.getName()) + SLASH_CHAR;
      }

      items.add(new BrowseListItem(link, node.getName(), !node.isLeaf(), lastModified, size, ""));
    }

    return items;
  }

  /**
   * Retrieves the asset associated with the external id.
   */
  private Optional<FluentAsset> getAssetById(final Repository repository, @Nullable final EntityId assetId) {
    if (assetId == null) {
      return Optional.empty();
    }
    return findContentFacets(repository)
        .map(ContentFacet::assets)
        .map(facet -> facet.find(assetId))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .findFirst();
  }

  /**
   * Get a link to the asset for use in the HTML view.
   */
  private String getAssetLink(final Repository repository, final Asset asset) {
    return repository.getUrl() + Stream.of(asset.path().split(SLASH)).map(escapeHelper::uri).collect(joining(SLASH));
  }
}
